document.addEventListener('DOMContentLoaded', function () {
  const form = document.getElementById('login-form');
  const messageDiv = document.getElementById('login-message');

  form.addEventListener('submit', async function (e) {
    e.preventDefault();

    // Get form data
    const email = form.email.value;
    const password = form.password.value;

    try {
      const response = await fetch('/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ email, password })
      });

      if (!response.ok) {
        let errorMsg = 'Login failed';
        try {
          const error = await response.json();
          errorMsg = error.message || errorMsg;
        } catch { }
        messageDiv.textContent = errorMsg;
        messageDiv.style.color = 'red';
        return;
      }

      const data = await response.json();
      const jwt = data.token || data.jwt; // adjust according to your API's response
      const username = data.username || data.username;

      if (!jwt) {
        messageDiv.textContent = 'Login failed: No token received.';
        messageDiv.style.color = 'red';
        return;
      }

      if (!username) {
        messageDiv.textContent = 'Login failed: No username.';
        messageDiv.style.color = 'red';
        return;
      }

      // Store JWT in localStorage
      localStorage.setItem('jwt', jwt);

      localStorage.setItem('username', username);

      messageDiv.textContent = 'Login successful!';
      messageDiv.style.color = 'green';

      // Redirect to homepage or update UI
      setTimeout(() => {
        window.location.href = '/web/index.html';
      }, 1000);

    } catch (err) {
      messageDiv.textContent = 'An error occurred. Please try again.';
      messageDiv.style.color = 'red';
    }
  });
});
