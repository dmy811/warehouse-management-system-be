#!/bin/bash
set -euo pipefail

DB_URL=${1:-"postgresql://postgres:postgress@localhost:5432/warehouse-axum"}

echo "Memulai seeders untuk database"
echo "=================================="

run_sql_file() {
    local file=$1
    local description=$2

    echo "Menjalankan: $description"
    if psql "$DB_URL" -f "$file" > /dev/null 2>&1; then
        echo "Berhasil: $description"
    else
        echo "Gagal: $description"
    fi
    echo ""
}

echo "Menjalankan Seeders..."
run_sql_file "seeders/001_insert_default_roles.sql" "Insert default roles"
run_sql_file "seeders/002_insert_users_admin.sql" "Insert users admin"

echo "Semua seeders berhasil diterapkan di database!"
echo ""
echo "Verifikasi data:"
psql "$DB_URL" -c "
SELECT
    u.id,
    u.name,
    u.email,
    r.name as role_name
FROM public.users u
LEFT JOIN public.user_roles ur ON u.id = ur.user_id
LEFT JOIN public.roles r ON ur.role_id = r.id
ORDER BY u.id;
"