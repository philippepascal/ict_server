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
# PUBLIC_KEY_CONTENT=$(awk '{ printf "%s", $0 }' "../target/${KEY_NAME}_pub.pem")
JSON_PAYLOAD=$(cat <<EOF
{
  "id": "$UUID",
  "pem_public_key": "$PUBLIC_KEY_CONTENT"
}
EOF
)

echo "payload $JSON_PAYLOAD"

# Step 6: POST to server
RES=$(curl -X POST "$SERVER_URL" \
     -H "Content-Type: application/json" \
     -d "$JSON_PAYLOAD" \
     -v)

echo -e "\n✅ Registration complete."
echo "res $RES"

# $(echo "$JSON_PAYLOAD" | jq -r '.encrypted_secret')

ENCODED_SECRET=$(echo "$RES" | jq -r '.encrypted_secret')

echo "encoded secret $ENCODED_SECRET"

# ENCRYPTED_SECRET=($(echo "$ENCODED_SECRET" | base64 -d | xxd -p -c1))

# ENCRYPTED_SECRET=$(echo "$ENCODED_SECRET" | base64 -d)

# echo "encrypted secret $ENCRYPTED_SECRET"

# DECRYPTED_SECRET=$(echo "$ENCODED_SECRET" | \
# base64 -d | xxd -p -c1 | \
# openssl pkeyutl -decrypt -inkey "../target/${KEY_NAME}_pkcs1.pem" ) 

# echo "decrypted secret $DECRYPTED_SECRET"

# DECRYPTED_SECRET=$(echo "$ENCODED_SECRET" | \
# base64 -d | xxd -p -c1 | \
# openssl pkeyutl -decrypt -inkey "../target/${KEY_NAME}_pkcs1.pem" | \
# awk 'BEGIN{RS="\0"} NR>1{print; exit}')  # skip padding, print real message

# echo "decrypted secret $DECRYPTED_SECRET"

DECRYPTED_SECRET=$(echo "$ENCODED_SECRET" | \
base64 -d | \
openssl pkeyutl -decrypt -inkey "../target/${KEY_NAME}_pkcs1.pem" ) 

echo "decrypted secret $DECRYPTED_SECRET"

# DECRYPTED_SECRET=$(echo "$ENCODED_SECRET" | \
# base64 -d | \
# openssl pkeyutl -decrypt -inkey "../target/${KEY_NAME}_pkcs1.pem" | \
# awk 'BEGIN{RS="\0"} NR>1{print; exit}')  # skip padding, print real message

# echo "decrypted secret $DECRYPTED_SECRET"

# B32=$(echo "$DECRYPTED_SECRET" | xxd -r -p | base32)

# B32=$(echo "$DECRYPTED_SECRET" | base32 -d)

# echo "$B32"

oathtool --totp -b "$DECRYPTED_SECRET"
