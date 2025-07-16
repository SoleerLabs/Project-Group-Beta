# 🛒 E-Commerce Backend API

A robust multi-vendor e-commerce backend API built with Rust, featuring comprehensive cart management, product catalog, and order processing capabilities.

## 🚀 Tech Stack

- **[Axum](https://github.com/tokio-rs/axum)** - Modern web framework for Rust
- **[SQLx](https://github.com/launchbadge/sqlx)** - Async SQL toolkit with compile-time checked queries
- **[Serde](https://serde.rs/)** - Serialization/deserialization framework
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime for Rust
- **[UUID](https://github.com/uuid-rs/uuid)** - UUID generation and parsing
- **[jsonwebtoken](https://github.com/Keats/jsonwebtoken)** - JWT implementation for authentication

## 📋 Features

### 🏪 Multi-Vendor Support
- Vendor registration and authentication
- Vendor-specific product management
- Vendor order tracking and status updates

### 🛍️ Product Management
- CRUD operations for products
- Advanced filtering and search capabilities
- Stock management
- Public product browsing

### 🛒 Shopping Cart
- Add/remove items from cart
- Quantity management
- Customer-specific cart persistence

### 📦 Order Processing
- Cart-to-order conversion
- Payment simulation
- Order status tracking
- Order history for customers and vendors

### 🔐 Authentication & Authorization
- JWT-based authentication
- Role-based access control (Vendor/Customer)
- Secure password handling

## 🗂️ Database Schema

The API uses a relational database with the following main entities:

- **Users** - Customer and vendor accounts
- **Products** - Product catalog with vendor associations
- **Cart Items** - Shopping cart management
- **Orders** - Order records and tracking
- **Order Items** - Individual items within orders
