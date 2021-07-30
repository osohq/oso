const API_BASE = 'https://docs-api.osohq.com';

function postIntegrationRequest(integration) {
  const url = `${API_BASE}/integration`;

  return fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      integration,
      origin: window.location.origin,
    }),
  });
}

function postFeedback(isUp, location) {
  const url = `${API_BASE}/feedback`;

  return fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      isUp,
      location: window.location.href,
    }),
  })
}

export {postIntegrationRequest, postFeedback};
