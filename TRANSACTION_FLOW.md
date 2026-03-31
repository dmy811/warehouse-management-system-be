# Transaction Flow

This document explains how each transaction type works in the WMS,
from creation to completion, and how it affects inventory.

---

## Goods Receipt (Inbound)

Triggered when goods arrive from a supplier into the warehouse.

```
Goods arrive from supplier
        │
        ▼
Staff creates Goods Receipt (DRAFT)
  → goods_receipts:      status = DRAFT
  → goods_receipt_items: records each product + quantity
                         (one row per product, multiple products allowed)
  → inventories:         not touched — will INSERT if this product has never been
                         in this warehouse before, or UPDATE quantity += x if it already exists,
                         but only after approval
        │
        ▼
Manager physically checks the goods
  → Does the quantity match? Is the condition acceptable?
        │
        ├── NO  → Edit or delete the DRAFT — inventories remains unchanged
        │
        ▼
Manager approves (DRAFT → RECEIVED)
  → goods_receipts:   status = RECEIVED
                      received_at = timestamp of approval
  → inventories:      for each item/product in goods_receipt_items:
                        if product never existed in this warehouse → INSERT new row (quantity = received qty)
                        if product already exists in this warehouse → UPDATE quantity += received qty
  → stock_movements:  for each item/product → INSERT one row
                        (direction = IN, reference_type = RECEIPT, reference_id = goods_receipt.id)
        │
        ▼
Staff allocates products to racks
  → inventory_racks:  staff decides which rack each product goes to
                      e.g. rack A1 gets 20 units, rack B2 gets 10 units
                      rule: SUM(inventory_racks.quantity) must equal inventories.quantity
```

---

## Shipment (Outbound)

Triggered when goods are dispatched from the warehouse to a customer.

```
Customer places an order
        │
        ▼
Staff creates Shipment (DRAFT)
  → shipments:       status = DRAFT
  → shipment_items:  records each product + quantity
                     (one row per product, multiple products allowed)
  → inventories:     not touched — will UPDATE quantity -= x per product,
                     but only after approval
        │
        ▼
Manager verifies stock availability and picking list
  → Is there enough stock for each product? Are the items and quantities correct?
        │
        ├── NO  → Edit or delete the DRAFT — inventories remains unchanged
        │
        ▼
Manager approves (DRAFT → SHIPPED)
  → shipments:       status = SHIPPED
  → inventories:     for each item/product in shipment_items:
                       UPDATE quantity -= shipped qty
                       (quantity cannot go below 0 — enforced by CHECK constraint)
  → stock_movements: for each item/product → INSERT one row
                       (direction = OUT, reference_type = SHIPMENT, reference_id = shipment.id)
  → inventory_racks: for each item/product → UPDATE quantity -= picked qty
                       from the specific racks the staff picked from
```

---

## Transfer (Inter-warehouse)

Triggered when goods are moved from one warehouse to another within the same system.
Unlike Goods Receipt and Shipment, a Transfer affects **two warehouses simultaneously**.

```
Warehouse A needs to send stock to Warehouse B
        │
        ▼
Staff or Manager creates Transfer (DRAFT)
  → transfers:       status = DRAFT
                     from_warehouse_id = A, to_warehouse_id = B
  → transfer_items:  records each product + quantity
                     (one row per product, multiple products allowed in one transfer)
  → inventories:     not touched on either warehouse —
                       Warehouse A: will UPDATE quantity -= x per product on approval
                       Warehouse B: will INSERT if product is new to Warehouse B,
                                    or UPDATE quantity += x if product already exists there,
                                    but only after approval
        │
        ▼
Manager verifies the transfer request
  → Is the stock in Warehouse A sufficient for each product?
        │
        ├── NO  → Edit or delete the DRAFT — inventories on both sides remains unchanged
        │
        ▼
Manager approves (DRAFT → COMPLETED)
  → transfers: status = COMPLETED
  │
  ├── Warehouse A (sender) — for each item/product in transfer_items:
  │     → inventories:      UPDATE quantity -= transferred qty
  │     → stock_movements:  INSERT one row
  │                           (direction = OUT, reference_type = TRANSFER, reference_id = transfer.id)
  │     → inventory_racks:  UPDATE quantity -= transferred qty from the source racks
  │
  └── Warehouse B (receiver) — for each item/product in transfer_items:
        → inventories:      if product never existed in Warehouse B → INSERT new row (quantity = transferred qty)
                            if product already exists in Warehouse B → UPDATE quantity += transferred qty
        → stock_movements:  INSERT one row
                              (direction = IN, reference_type = TRANSFER, reference_id = transfer.id)
        │
        ▼
Staff in Warehouse B allocates received products to racks
  → inventory_racks: assign each product to specific racks in Warehouse B
                     rule: SUM(inventory_racks.quantity) must equal inventories.quantity
```

---

## Stock Movements (Audit Log)

`stock_movements` is an **append-only audit log**. It is never written to directly by users —
every record is created automatically by the system when a transaction is approved.

```
Goods Receipt → RECEIVED
  → INSERT stock_movements:
      product_id     = item.product_id
      warehouse_id   = goods_receipt.warehouse_id
      direction      = IN
      quantity       = item.quantity
      reference_type = RECEIPT
      reference_id   = goods_receipt.id
      (one row inserted per product in goods_receipt_items)

Shipment → SHIPPED
  → INSERT stock_movements:
      product_id     = item.product_id
      warehouse_id   = shipment.warehouse_id
      direction      = OUT
      quantity       = item.quantity
      reference_type = SHIPMENT
      reference_id   = shipment.id
      (one row inserted per product in shipment_items)

Transfer → COMPLETED
  → INSERT stock_movements (sender side):
      product_id     = item.product_id
      warehouse_id   = transfer.from_warehouse_id
      direction      = OUT
      quantity       = item.quantity
      reference_type = TRANSFER
      reference_id   = transfer.id

  → INSERT stock_movements (receiver side):
      product_id     = item.product_id
      warehouse_id   = transfer.to_warehouse_id
      direction      = IN
      quantity       = item.quantity
      reference_type = TRANSFER
      reference_id   = transfer.id

  (two rows inserted per product in transfer_items — one for each warehouse)
```

No role can edit or delete records from this table. This guarantees a complete and
tamper-proof history of every inventory change.

**What stock_movements can answer:**

- Why did the stock for product X change on date Y?
- What is the total goods received from all suppliers this month?
- What is the total goods shipped to all customers this quarter?
- Full movement history for a specific product in a specific warehouse

**Reconstructing current inventory from scratch using only stock_movements:**

```sql
SELECT
    product_id,
    warehouse_id,
    SUM(CASE WHEN direction = 'IN'  THEN quantity ELSE 0 END) -
    SUM(CASE WHEN direction = 'OUT' THEN quantity ELSE 0 END) AS current_stock
FROM stock_movements
GROUP BY product_id, warehouse_id;

-- This result must always match inventories.quantity for every product + warehouse combination.
-- If it does not match, there is a data integrity bug in the system.
```

---

## Inventory Consistency Rules

At all times, the following two rules must hold true across the entire system:

**Rule 1 — stock_movements must match inventories**

```
For every (product_id, warehouse_id) combination:

  SUM(stock_movements.quantity WHERE direction = 'IN')
- SUM(stock_movements.quantity WHERE direction = 'OUT')

  must equal

  inventories.quantity
```

**Rule 2 — inventory_racks must match inventories**

```
For every inventory record:

  SUM(inventory_racks.quantity WHERE inventory_id = x)

  must equal

  inventories.quantity WHERE id = x
```

If either rule is violated, it indicates a data integrity issue in the system.
