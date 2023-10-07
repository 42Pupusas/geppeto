var { getSharedSecret, schnorr, utils } = nobleSecp256k1;
var crypto = window.crypto;
var getRand = (size) => crypto.getRandomValues(new Uint8Array(size));
var sha256 = bitcoinjs.crypto.sha256;


document.addEventListener('DOMContentLoaded', function() {
    document.body.addEventListener('htmx:load', function(evt) {
        if (evt.target.id === "didSend") {
            console.log("Sent");
            setTimeout(function() {
                // Get the div by its id.
                var didSendDiv = document.getElementById("didSend");

                // Set the opacity of the div to 0, triggering the transition.
                didSendDiv.style.opacity = "0";
            }, 2000); // 2000ms = 2s
        }
    });
});

function createNote(content, kind, tags) {
    let created_at = Math.floor(Date.now() / 1000);
    let note = {
        "content": content,
        "tags": tags,
        "kind": kind,
        "created_at": created_at,
    };
    return note;
}

async function getSignedNote(event) {
    var privateKey = window.keypair.privateKey;
    var publickKey = window.keypair.publicKey.toString("hex").substring(2);

    var eventData = JSON.stringify([
        0, // Reserved for future use
        publickKey, // The sender's public key
        event["created_at"], // Unix timestamp
        event["kind"], // Message “kind” or type
        event["tags"], // Tags identify replies/recipients
        event["content"], // Your note contents
    ]);
    event.pubkey = publickKey;
    event.id = sha256(eventData).toString("hex");
    event.sig = await schnorr.sign(event.id, privateKey);
    return event;
}

function createNewPrivateKey() {

    try {
        const keys = document.getElementById('keys');
        var key = bitcoinjs.ECPair.makeRandom().privateKey.toString('hex');
        keys.innerText = key;
    } catch (error) {
        console.error("Failed to create private key:", error);
    }
}

async function checkInToNostr() {
    try {
        const checkIn = document.getElementById("checkInRequest");
        let newNote = createNote("Checking In", 20420, []);
        let signedNote = await window.nostr.signEvent(newNote);
        const publicKey = await window.nostr.getPublicKey();
        checkIn.setAttribute('hx-vals', JSON.stringify(signedNote));
        window.publicKey = publicKey;
        checkIn.dispatchEvent(new Event("change"));
    } catch (error) {
        console.error("Failed to retrieve public key:", error);
    }
}

async function checkInWithHex() {
    try {
        const checkIn = document.getElementById("checkInRequest");
        const privateKey = document.getElementById("privateKey").value;
        window.keypair = bitcoinjs.ECPair.fromPrivateKey(Buffer.from(privateKey, "hex"));
        window.publicKey = window.keypair.publicKey.toString("hex").substring(2);
        window.publicKey = publicKey;
        console.log(window.keypair);
        let newNote = createNote("Checking In", 20420, []);
        let signedNote = await getSignedNote(
            newNote,
            window.privateKey
        );
        console.log(signedNote);
        console.log(publicKey);
        checkIn.setAttribute('hx-vals', JSON.stringify(signedNote));
        checkIn.dispatchEvent(new Event("change"));
        console.log("Check in sent");
    } catch (error) {
        console.error("Failed to retrieve public key:", error);
    }
}

async function sendNoteWithNip() {
    try {
        const sendNoteToNostr = document.getElementById("sendNoteRequest");
        let newNoteContent = document.getElementById("newNoteContent").value;
        let newNoteKind = document.getElementById("newNoteKind").value / 1;
        let toRelay = document.getElementById("toRelay").value;
        let newNote = createNote(newNoteContent, newNoteKind, []);
        let signedNote = await window.nostr.signEvent(newNote);
        let json_response = { relay: toRelay, note: JSON.stringify(signedNote) };
        sendNoteToNostr.setAttribute('hx-vals', JSON.stringify(json_response));
        sendNoteToNostr.dispatchEvent(new Event("change"));
    } catch (error) {
        console.error("Failed to create note:", error);
    }
}

async function sendNoteWithHex() {
    try {
        const sendNoteToNostr = document.getElementById("sendNoteRequest");
        let newNoteContent = document.getElementById("newNoteContent").value;
        let newNoteKind = document.getElementById("newNoteKind").value / 1;
        let toRelay = document.getElementById("toRelay").value;
        let newNote = createNote(newNoteContent, newNoteKind, []);
        let signedNote = await getSignedNote(
            newNote,
            window.privateKey
        );
        let json_response = { relay: toRelay, note: JSON.stringify(signedNote) };
        sendNoteToNostr.setAttribute('hx-vals', JSON.stringify(json_response));
        sendNoteToNostr.dispatchEvent(new Event("change"));
    } catch (error) {
        console.error("Failed to create note:", error);
    }
}
