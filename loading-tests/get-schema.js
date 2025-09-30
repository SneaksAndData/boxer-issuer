// k6 script to load test the root token and schema endpoints
//
//
// Usage example:
//
// export ROOT_TOKEN_ENDPOINT="https://example.com/get-root-token"
// export ROOT_SCHEMA_ENDPOINT="https://example.com/schema"
// export ROOT_EXTERNAL_TOKEN="staticBearerToken"
// SCHEMA_CALLS=10 k6 run loading-tests/get-schema.js

import http from 'k6/http';
import {check, fail, sleep} from 'k6';

export const options = {
    vus: 1,
    duration: '10s',
};

const ROOT_TOKEN_URL = __ENV.ROOT_TOKEN_ENDPOINT
const SCHEMA_URL = __ENV.ROOT_SCHEMA_ENDPOINT

const ROOT_STATIC_BEARER = __ENV.ROOT_EXTERNAL_TOKEN
const SCHEMA_CALLS = parseInt(__ENV.SCHEMA_CALLS || '5', 10);

export function setup() {
    const rootRes = http.get(ROOT_TOKEN_URL, {
        headers: ROOT_STATIC_BEARER ? {Authorization: `Bearer ${ROOT_STATIC_BEARER}`} : {},
    });

    check(rootRes, {
        'root token status 200': r => r.status === 200,
        'root token body not empty': r => (r.body || '').length > 0,
    }) || fail('Failed to retrieve root token');

    const token = (rootRes.body || '').trim();
    if (!token) fail('Token empty');

    return {token};
}

export default function (data) {
    for (let i = 0; i < SCHEMA_CALLS; i++) {
        const res = http.get(SCHEMA_URL, {
            headers: {
                Authorization: `Bearer ${data.token}`,
                Accept: 'application/json',
            },
        });

        check(res, {
            'schema status 200': r => r.status === 200,
            'schema body present': r => (r.body || '').length > 0,
        });

        sleep(0.2);
    }
}
