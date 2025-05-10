document.addEventListener('DOMContentLoaded', function () {
  const form = document.getElementById('signup-form');
  const messageDiv = document.getElementById('signup-message');

  form.addEventListener('submit', async function (e) {
    e.preventDefault();

    // Get form data
    const email = form.email.value;
    const password = form.password.value;
    // Add other fields as needed (e.g., username, but do not expect it back)

    try {
      const response = await fetch('/api/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ email, password /*, username: form.username.value */ })
      });

      if (!response.ok) {
        let errorMsg = 'Signup failed';
        try {
          const error = await response.json();
          errorMsg = error.message || errorMsg;
        } catch { }
        messageDiv.textContent = errorMsg;
        messageDiv.style.color = 'red';
        return;
      }

      const data = await response.json();
      const jwt = data.token || data.jwt;

      if (!jwt) {
        messageDiv.textContent = 'Signup failed: No token received.';
        messageDiv.style.color = 'red';
        return;
      }

      // Store JWT in localStorage
      localStorage.setItem('jwt', jwt);

      messageDiv.textContent = 'Signup successful!';
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
