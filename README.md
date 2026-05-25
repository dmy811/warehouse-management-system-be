# Role-Based Access Control (RBAC)

This document describes the access control system for the Warehouse Management System (WMS).
There are 4 roles: **ADMIN**, **MANAGER**, **STAFF**, and **VIEWER**.

---

## Role Overview

| Role       | Description                                                                        |
| ---------- | ---------------------------------------------------------------------------------- |
| 👑 ADMIN   | Full access to all features. Intended for IT admin or director of operations.      |
| 🏢 MANAGER | Manages day-to-day warehouse operations. Cannot manage users or system config.     |
| 📦 STAFF   | Warehouse operator. Can only input DRAFT transactions in their assigned warehouse. |
| 👁️ VIEWER  | Read-only. For auditors, accountants, or management who only need reports.         |

---

## Access Matrix

| Module              | ADMIN                               | MANAGER                  | STAFF                | VIEWER    |
| ------------------- | ----------------------------------- | ------------------------ | -------------------- | --------- |
| User Management     | Full CRUD + assign role & warehouse | Read only                | ❌                   | ❌        |
| Warehouse           | Full CRUD                           | Read only                | Read only            | Read only |
| Product & Category  | Full CRUD                           | CRU                      | Read only            | Read only |
| Supplier & Customer | Full CRUD                           | CRU                      | Read only            | Read only |
| Rack                | Full CRUD                           | CRU                      | Read only            | Read only |
| Goods Receipt       | Full CRUD + approve                 | CRU, approve             | Create (DRAFT), read | Read only |
| Shipment            | Full CRUD + approve                 | CRU, approve             | Create (DRAFT), read | Read only |
| Transfer            | Full CRUD + approve                 | CRU, approve             | Create (DRAFT), read | Read only |
| Inventory           | Read + manual adjustment            | Read + manual adjustment | Read only            | Read only |
| Stock Movement      | Read only                           | Read only                | Read only            | Read only |
| Report              | Full access                         | Full access              | Own warehouse only   | Read only |

---

## Transaction Status Flow

Each transaction (Goods Receipt, Shipment, Transfer) follows a strict status flow with role-based approval.

```
Goods Receipt:   DRAFT ──────────────────► RECEIVED
Shipment:        DRAFT ──────────────────► SHIPPED
Transfer:        DRAFT ──────────────────► COMPLETED
```

| Status    | Who can set it        | Description                                |
| --------- | --------------------- | ------------------------------------------ |
| DRAFT     | ADMIN, MANAGER, STAFF | Initial state when transaction is created  |
| RECEIVED  | ADMIN, MANAGER only   | Goods Receipt has been physically received |
| SHIPPED   | ADMIN, MANAGER only   | Shipment has been dispatched to customer   |
| COMPLETED | ADMIN, MANAGER only   | Transfer between warehouses is done        |

> **Why this separation exists:**
> STAFF creates the transaction (data entry), MANAGER or ADMIN verifies and approves it.
> This creates a clear audit trail — there is always a separation between the person who inputs and the person who approves.

---

## Warehouse Access Restriction (STAFF only)

STAFF is restricted to the warehouse(s) they are assigned to via the `user_warehouses` table.
This restriction applies to all data they can read and all transactions they can create.

```
user_warehouses
├── user_id      → references users.id
└── warehouse_id → references warehouses.id

A STAFF member assigned to Warehouse A:
  ✅ Can create a Goods Receipt for Warehouse A
  ❌ Cannot see or create transactions for Warehouse B
```

ADMIN and MANAGER have access to all warehouses without restriction.

---

## Stock Movement — Append Only

`stock_movements` is an **immutable audit log**. No role can edit or delete records from this table.
Every change to inventory quantity automatically creates a new record here.

| Event                           | direction | reference_type |
| ------------------------------- | --------- | -------------- |
| Goods Receipt → RECEIVED        | IN        | RECEIPT        |
| Shipment → SHIPPED              | OUT       | SHIPMENT       |
| Transfer → COMPLETED (sender)   | OUT       | TRANSFER       |
| Transfer → COMPLETED (receiver) | IN        | TRANSFER       |

---

## Implementation Reference

Role constants are defined in `src/constants.rs`:

```rust
pub mod roles {
    pub const ADMIN: &str   = "ADMIN";
    pub const MANAGER: &str = "MANAGER";
    pub const STAFF: &str   = "STAFF";
    pub const VIEWER: &str  = "VIEWER";
}

pub mod permissions {
    use super::roles::*;

    pub const CAN_APPROVE: &[&str]             = &[ADMIN, MANAGER];
    pub const CAN_MANAGE_MASTER: &[&str]       = &[ADMIN, MANAGER];
    pub const CAN_MANAGE_USERS: &[&str]        = &[ADMIN];
    pub const CAN_CREATE_TRANSACTION: &[&str]  = &[ADMIN, MANAGER, STAFF];
    pub const ALL_ROLES: &[&str] = &[ADMIN, MANAGER, STAFF, VIEWER];
}
```

Role enforcement in handlers via `require_roles`:

```rust
// Only ADMIN can delete a warehouse
pub async fn delete(State(state): State<AppState>, Extension(auth_user): Extension<AuthUser>, ...) {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    ...
}

// ADMIN and MANAGER can approve a goods receipt
pub async fn delete(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> AppResult<impl axum::response::IntoResponse> {
    require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    ...
}
```

Warehouse access check for STAFF in the service layer:

```rust
if auth_user.role == roles::STAFF {
    let has_access = user_warehouse_repo
        .check_access(auth_user.id, warehouse_id)
        .await?;

    if !has_access {
        return Err(AppError::Forbidden);
    }
}
```

##### Database diagram

![diagram](./public/wms-database-diagram.png)
