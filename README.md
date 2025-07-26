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


E-commerce Team Database Setup Guide
🎯 Overview
This project uses a shared Render PostgreSQL database for team collaboration. All team members connect to the same database instance, allowing real-time collaboration and testing.
⚠️ IMPORTANT LIMITATIONS

🕐 Free tier expires in 30 days - Plan accordingly for assignment deadline
💾 1GB storage limit on free tier
🗑️ Database will be DELETED after 30 days
📅 Expiration Date: [ADD YOUR EXPIRATION DATE HERE]

🚀 Quick Start for New Team Members
Step 1: Clone and Setup
# Clone the repository
git clone [YOUR_REPO_URL]
cd [YOUR_PROJECT_NAME]

Step 2: Get Database Credentials
Contact team lead for the complete .env file with database credentials.
Your .env should look like this:
DATABASE_URL=postgresql://ecommerce_ffqm_user:PASSWORD@dpg-xxxxx-a.oregon-postgres.render.com:5432/ecommerce_ffqm
JWT_SECRET=your-shared-jwt-secret
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

Step 3: Install Dependencies
# Install Rust dependencies
cargo build

# Install PostgreSQL client (for database access)
# macOS:
brew install postgresql

# Ubuntu/Debian:
sudo apt install postgresql-client

# Windows:
# Download from https://www.postgresql.org/download/windows/

Step 4: Test Database Connection
# Test connection
cargo run

# You should see:
# ✅ Successfully connected to Render PostgreSQL!

🗄️ Database Connection Details
Connection Information

Provider: Render PostgreSQL (Free Tier)
Database Name: ecommerce_ffqm
Host: dpg-d1tnq3jipnbc73chnrag-a.oregon-postgres.render.com
Port: 5432
SSL: Required (automatically handled)

Connection Methods
Method 1: From Rust Application:
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url).await?;
    // Your code here
    Ok(())
}
Method 2: Direct psql Connection:
# Connect via command line
psql "postgresql://ecommerce_ffqm_user:PASSWORD@dpg-xxxxx-a.oregon-postgres.render.com:5432/ecommerce_ffqm"

Method 3: Database GUI Client
Use DBeaver, pgAdmin, or TablePlus with the connection details above.

Verify Setup:
-- Check if tables exist
\dt

-- View sample data
SELECT * FROM users;
SELECT * FROM products LIMIT 5;

Development Process

Pull latest changes: git pull origin main
Create feature branch: git checkout -b feature/your-feature-name
Develop and test using shared database
Push and create PR: git push origin feature/your-feature-name
Team review and merge
