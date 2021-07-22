const API_BASE = 'https://docs-api.osohq.com';

function postIntegrationRequest(integration) {
  const url = `${API_BASE}/integration`;

  fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({integration}),
  });
}

export {postIntegrationRequest};
