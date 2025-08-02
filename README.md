# Identity-based Control Trigger (ICT)

A lightweight, secure web server running on a Raspberry Pi to **authenticate users and activate pins via GPIO**.  
ICT uses **identity-based authentication (RSA keys + TOTP)** to ensure only authorized devices can trigger actions. Activated pins can be used to operate relay or other binary type of actions. Command line arguments will refer to relay instead of pins as this was my use case with coding this.

---

## ✨ Features
- 🔒 **Secure authentication** with RSA keys and TOTP  
- ⚡ **Relay control** via Raspberry Pi GPIO  
- 🌐 **REST API** for easy integration  
- 📜 **Audit logging** of all actions  
- 🛠 **Lightweight and fast** (built in Rust)  

## Important Notes
- This is not a production ready project. Be aware that a lot of elbow grease is still needed to properly and securely install and run on a Pi (nginx, logrotate, service configuration, firewall, to name a few). I provide a few examples, but this is highly dependant on your use case.
- The security claims are my opinion only, use at your own risk. That said, if anyone uses this and find bugs/security risk, please let me know or create a pull request to address the issue.

---

## 📋 To-Do List
- Replace `rouille` with `tiny_http` (to remove deprecated dependencies: `buf_redux`, `multipart`)  

---


## Design

- A client needs to first create for itself a UUID and a pair of public/private key. Using a secure enclave for the private key if possible.
- The client then calls the register action, passing its UUID and public key.
- The server will store those, but at this point the client is neither authorized nor has relays associated with it.
- The server will then respond with a secret (encrypted with the public key) to be used later by TOTP.

- An admin needs to use the command line (same executable) on the server to associate relays and authorize the client. There is no public API to do this by design: it's simpler and safer for my use case.

- Once that is done, the client operate a relay by calling the operate action, passing UUID, TOTP and salt in a token signed by the private key.
- Upon receiving the operate call, the server first verify the signature, then the TOTP, and actuate the relay for a duration specified in the configuration.

### 📜 Register Action

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant DB as Database

    Client->>Server: POST /register (uuid, public_key)
    Server->>DB: Store (uuid, public_key, secret)
    DB-->>Server: Success
    Server-->>Client: 201 Created (encrypted secret)
```

### 📜 Associate Relay Action

```mermaid
sequenceDiagram
    participant Command-Line
    participant DB as Database

    Command-Line->>DB: Store (uuid, relay)
    DB-->>Command-Line: Success
```

### 📜 Authorize Action

```mermaid
sequenceDiagram
    participant Command-Line
    participant DB as Database

    Command-Line->>DB: Store (uuid, authorized flag)
    DB-->>Command-Line: Success
```

### ⚡ Operate Action

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant GPIO as Relay GPIO

    Client->>Server: POST /open (Authorization: signed (UUID, TOTP, salt))
    Server->>Server: Validate Signature and TOTP
    alt Valid
        Server->>GPIO: Trigger relays
        GPIO-->>Server: OK
        Server-->>Client: 200 OK (relay opened)
    else Invalid
        Server-->>Client: 401 Unauthorized
    end
```

---

## Help overview

```bash
Usage: ict_server [OPTIONS] <COMMAND>

Commands:
  register         Register a new client with the server
  authorize        Authorize a previously registered client
  unauthorize      Temporarily un-authorize a client (can be re-authorized)
  delete           Permanently delete a client
  operate          Operate client's relays after message validation
  list-clients     Lists all clients
  describe-client  Displays info and status of a client
  associate-relay  Associates a relay with a client
  clear-relays     Removes all relay of a client
  serve            Starts Web Server listening for clients
  help             Print this message or the help of the given subcommand(s)

Options:
  -c, --config <PATH-TO-FILE>  [default: configs/ict_server.toml]
  -h, --help                   Print help
```

---


## Files

```
ict_server/
├── Cargo.toml           # Project dependencies and metadata
├── README.md            # Project documentation
├── configs/
│   └── ict_server.toml  # Default configuration file
│   └── ict_server_sha1.toml  # Test configuration file used with tests.sh. use SHA1 as oathtool might not support SHA256 on pi os.
├── src/                 # Rust source code
│   └── ...              # 
├── tests/               # Tests code
│   ├── tests.sh         # a bash script to test against a running server
│   └── ...              # rust functional tests
└── ...
```



---

## 🛠 Handy Commands

```bash
# Display comprehensive help
cargo run -- help                                                                                                                                                          

# Run tests with logs
RUST_LOG=info cargo test -- --nocapture

# Run with GPIO feature (may require sudo)
cargo run --features gpio -- serve -p 3456

# Run specific test
RUST_LOG=info cargo test --features gpio -- test_happy_path --nocapture

# Shutdown Pi
sudo shutdown -h now

# Authorize a device
cargo run --features gpio -- authorize
cargo run -- authorize -u E791366E-40CE-4F85-8F92-8B7E6185EDC

# Associate a relay
cargo run -- associate-relay -r 10 -u E791366E-40CE-4F85-8F92-8B7E6185EDC1

# test pins 
pinctrl
pinctrl poll 16,20,21
```
