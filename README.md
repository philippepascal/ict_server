# Identity-based Control Trigger (ICT)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

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

---

# Apache License 2.0

                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

   END OF TERMS AND CONDITIONS

   Copyright 2025 Philippe Pascal

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
