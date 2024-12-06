import { check, sleep } from 'k6';
import http from 'k6/http';


export let options = {
    vus: 300, // Number of virtual users
    duration: '1000s', // Duration of the test
};

const base32Chars = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';

// Function to convert a number to Base32
function toBase32(num) {
    let base32 = '';
    while (num > 0) {
        base32 = base32Chars[num % 32] + base32;
        num = Math.floor(num / 32);
    }
    return base32.padStart(10, '0'); // Pad to 10 characters
}

function generateULID() {
    const timestamp = Date.now();
    const timePart = toBase32(Math.floor(timestamp / 1000)); // Convert to seconds and Base32

    // Generate 16 random characters for the random part
    let randomPart = '';
    for (let i = 0; i < 16; i++) {
        const randomByte = Math.floor(Math.random() * 32); // Generate a random number between 0 and 31
        randomPart += base32Chars[randomByte]; // Use Base32 characters
    }

    return timePart + randomPart; // Combine time and random parts
}

// Function to get a random event type
function getRandomEventType() {
    const eventTypes = ['login', 'purchase', 'logout'];
    return eventTypes[Math.floor(Math.random() * eventTypes.length)];
}

// Function to get a random payload
function getRandomPayload() {
    const payloads = ['User logged in', 'User made a purchase', 'User logged out'];
    return payloads[Math.floor(Math.random() * payloads.length)];
}

// Function to query events
function queryEvents() {
    const eventType = getRandomEventType(); // Get a random event type for querying
    let res = http.get(`http://localhost:3030/query?event_type=${eventType}`);
    check(res, {
        'is status 200': (r) => r.status === 200,
    });
}

// Function to ingest an event
function ingestEvent() {
    const payload = JSON.stringify({
        event_id: generateULID(), // Generate a random ULID
        event_type: getRandomEventType(), // Get a random event type
        timestamp: Date.now(), // Current timestamp
        payload: getRandomPayload(), // Get a random payload
    });

    const params = {
        headers: {
            'Content-Type': 'application/json',
        },
    };

    let res = http.post('http://localhost:3030/ingest', payload, params);
    check(res, {
        'is status 200': (r) => r.status === 200,
    });
}

export default function () {
    const randNum = Math.random();
    if (randNum < 0.2) {
        // 20% chance to ingest an event
        ingestEvent();
    } else {
        // 80% chance to query events
        ingestEvent();
    }

    sleep(1); // Sleep for 1 second between requests
}