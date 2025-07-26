-- E-Commerce Backend Database Schema
-- Complete schema for multi-vendor e-commerce with users, products, cart, and orders

-- USERS TABLE (Authentication & Authorization)

CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(225) DEFAULT 'customer', -- 'customer' or 'vendor'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


-- PRODUCTS TABLE (Vendor Inventory)

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10,2) NOT NULL CHECK (price >= 0),
    stock INTEGER NOT NULL DEFAULT 0 CHECK (stock >= 0),
    category VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- CART SYSTEM (Customer Shopping Cart)

CREATE TABLE cart_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, product_id) -- One cart item per product per user
);

-- ORDERS SYSTEM (Customer Orders)

CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total DECIMAL(10, 2) NOT NULL CHECK (total >= 0),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- 'pending', 'shipped', 'delivered'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ORDER ITEMS (Products in Each Order)

CREATE TABLE order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    vendor_id UUID NOT NULL REFERENCES users(id),  -- For vendor-specific queries
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    price DECIMAL(10, 2) NOT NULL CHECK (price >= 0), -- Price snapshot at purchase time
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- PERFORMANCE INDEXES

-- Product indexes (for searching and vendor queries)
CREATE INDEX idx_products_vendor_id ON products(vendor_id);
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_price ON products(price);
CREATE INDEX idx_products_name ON products(name);

-- Cart indexes (for fast cart operations)
CREATE INDEX idx_cart_items_user_id ON cart_items(user_id);
CREATE INDEX idx_cart_items_product_id ON cart_items(product_id);

-- Order indexes (for customer and vendor order queries)
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);

-- Order items indexes (for vendor-specific order filtering)
CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_vendor_id ON order_items(vendor_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);