#!/bin/bash

#openssl genpkey -algorithm RSA -out private_pkcs1.pem -pkeyopt rsa_keygen_bits:2048
#openssl pkcs8 -topk8 -inform PEM -in private_pkcs1.pem -outform PEM -nocrypt -out private_pkcs8.pem
#openssl pkey -in private_pkcs8.pem -pubout -out public.pem

# Configuration
SERVER_URL="http://localhost:3456/register"   # ← Replace with your actual server URL
KEY_NAME="mykey"                                # base filename
KEY_BITS=2048

# Step 1: Generate RSA private key (PKCS#1)
openssl genpkey -algorithm RSA -out "../target/${KEY_NAME}_pkcs1.pem" -pkeyopt rsa_keygen_bits:$KEY_BITS

# Step 2: Convert to PKCS#8 (PEM, unencrypted)
openssl pkcs8 -topk8 -inform PEM -in "../target/${KEY_NAME}_pkcs1.pem" -outform PEM -nocrypt -out "../target/${KEY_NAME}_pkcs8.pem"

# Step 3: Extract public key (PEM)
openssl pkey -in "../target/${KEY_NAME}_pkcs8.pem" -pubout -out "../target/${KEY_NAME}_pub.pem"

# Step 4: Generate a random UUID
UUID=$(uuidgen)

# Step 5: Prepare the JSON body
PUBLIC_KEY_CONTENT=$(awk '{ printf "%s\\n", $0 }' "../target/${KEY_NAME}_pub.pem")
JSON_PAYLOAD=$(cat <<EOF
{
  "id": "$UUID",
  "pem_public_key": "$PUBLIC_KEY_CONTENT"
}
EOF
)

echo $JSON_PAYLOAD

# Step 6: POST to server
curl -X POST "$SERVER_URL" \
     -H "Content-Type: application/json" \
     -d "$JSON_PAYLOAD" \
     -v

echo -e "\n✅ Registration complete."
