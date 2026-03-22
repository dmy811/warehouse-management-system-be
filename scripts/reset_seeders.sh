#!/bin/bash
set -euo pipefail

DB_URL=${1:-"postgresql://postgres:postgress@localhost:5432/warehouse-axum"}

echo "Memulai reset seeders di database"
echo "=================================="

run_sql_file() {
    local file=$1
    local description=$2

    echo "Menjalankan: $description"
    psql "$DB_URL" -f "$file" > /dev/null
    echo "Berhasil: $description"
    echo ""
}

echo "Menghapus data lama..."

psql "$DB_URL" -c "
TRUNCATE TABLE
    public.user_roles,
    public.users,
    public.roles
RESTART IDENTITY CASCADE;
" > /dev/null

echo "Berhasil menghapus data lama"
echo ""

echo "Menjalankan Seeders..."

run_sql_file "seeders/001_insert_default_roles.sql" "Insert default roles"
run_sql_file "seeders/002_insert_sample_users.sql" "Insert default users"

echo "Reset seeders berhasil diterapkan di database!"
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