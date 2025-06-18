#!/bin/bash

#this test requires the server to run with Sha1 totp. 
#from the tests directory, run:  "cargo run -- -c ict_server.toml serve -p 3456"

#openssl genpkey -algorithm RSA -out private_pkcs1.pem -pkeyopt rsa_keygen_bits:2048
#openssl pkcs8 -topk8 -inform PEM -in private_pkcs1.pem -outform PEM -nocrypt -out private_pkcs8.pem
#openssl pkey -in private_pkcs8.pem -pubout -out public.pem

# Defaults
HOST="localhost"
UUID=""
PAUSE=""

# Parse options
while getopts "h:u:a" opt; do
  case $opt in
    h)
      HOST="$OPTARG"
      ;;
    u)
      UUID="$OPTARG"
      ;;
    a)
      PAUSE="Y"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      ;;
  esac
done

echo "HOST is: $HOST"


# Configuration
SERVER_URL="http://$HOST:3456/register"   # ← Replace with your actual server URL
SERVER_URL2="http://$HOST:3456/operate"   # ← Replace with your actual server URL
KEY_NAME="mykey"                                # base filename
KEY_BITS=2048

# Step 1: Generate RSA private key (PKCS#1)
openssl genpkey -algorithm RSA -out "../target/${KEY_NAME}_pkcs1.pem" -pkeyopt rsa_keygen_bits:$KEY_BITS

# Step 2: Convert to PKCS#8 (PEM, unencrypted)
openssl pkcs8 -topk8 -inform PEM -in "../target/${KEY_NAME}_pkcs1.pem" -outform PEM -nocrypt -out "../target/${KEY_NAME}_pkcs8.pem"

# Step 3: Extract public key (PEM)
openssl pkey -in "../target/${KEY_NAME}_pkcs8.pem" -pubout -out "../target/${KEY_NAME}_pub.pem"

# Step 4: Generate a random UUID
if [ -z "$UUID" ]; then
  UUID=$(uuidgen)
fi

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


DECRYPTED_SECRET=$(echo "$ENCODED_SECRET" | \
base64 -d | \
openssl pkeyutl -decrypt -inkey "../target/${KEY_NAME}_pkcs1.pem" ) 

echo "decrypted secret $DECRYPTED_SECRET"

#sha256 not supported. for this to work, temp change to sha 1 in rust
# TOKEN=$(oathtool --totp --sha256 -b --now $(( $(date +%s) + 30 )) "$DECRYPTED_SECRET")
TOKEN=$(oathtool --totp -b "$DECRYPTED_SECRET")

echo "token $TOKEN"

SALT="random_salt_xyz"

JSON_PAYLOAD=$(jq -n \
  --arg token "$TOKEN" \
  --arg _salt "$SALT" \
  '{token: $token, _salt: $_salt}')

echo "payload $JSON_PAYLOAD"

SIGNATURE=$(printf '%s' "$JSON_PAYLOAD" | \
  openssl dgst -sha256 -sign "../target/${KEY_NAME}_pkcs8.pem" | \
  base64)

echo "signature $SIGNATURE"

FULL_PAYLOAD=$(jq -n \
--arg id "$UUID" \
--arg totp_message "$JSON_PAYLOAD" \
--arg signature "$SIGNATURE" \
'{id: $id, totp_message: $totp_message, signature: $signature}')

echo "full payload $FULL_PAYLOAD"

#this one is supposed to fail (not authorized)
curl -X POST "$SERVER_URL2" \
     -H "Content-Type: application/json" \
     -d "$FULL_PAYLOAD" \
     -v

if [ -z "$PAUSE" ]; then
  cargo run -- -c ict_server.toml authorize -u $UUID
else
  read -n 1 -s -r -p "Paused to allow you to authorize manually with UUID $UUID. Press any key to continue..."
  echo
fi

#recreate a token in case authorization was manual and took too much time
TOKEN=$(oathtool --totp -b "$DECRYPTED_SECRET")

echo "token $TOKEN"

SALT="random_salt_xyz"

JSON_PAYLOAD=$(jq -n \
  --arg token "$TOKEN" \
  --arg _salt "$SALT" \
  '{token: $token, _salt: $_salt}')

echo "payload $JSON_PAYLOAD"

SIGNATURE=$(printf '%s' "$JSON_PAYLOAD" | \
  openssl dgst -sha256 -sign "../target/${KEY_NAME}_pkcs8.pem" | \
  base64)

echo "signature $SIGNATURE"

FULL_PAYLOAD=$(jq -n \
--arg id "$UUID" \
--arg totp_message "$JSON_PAYLOAD" \
--arg signature "$SIGNATURE" \
'{id: $id, totp_message: $totp_message, signature: $signature}')

echo "full payload $FULL_PAYLOAD"

curl -X POST "$SERVER_URL2" \
     -H "Content-Type: application/json" \
     -d "$FULL_PAYLOAD" \
     -v