document.addEventListener('DOMContentLoaded', function () {
  const form = document.getElementById('signup-form');
  const messageDiv = document.getElementById('signup-message');

  form.addEventListener('submit', async function (e) {
    e.preventDefault();

    // Get form data
    const firstname = form.firstname.value;
    const lastname = form.lastname.value;
    const username = form.username.value;
    const email = form.email.value;
    const password = form.password.value;
    // Add other fields as needed (e.g., username, but do not expect it back)

    try {
      const response = await fetch('/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ firstname, lastname, username, email, password })
      });


      if (!response.ok) {
        let errorMsg = 'Signup failed';
        try {
          const error = await response.json();
          errorMsg = error.message || errorMsg;
        } catch { }
        messageDiv.textContent = errorMsg;
        messageDiv.classList.remove('hidden'); // Ensure message is visible
        messageDiv.classList.add('text-red-600'); // Style for error messages
        return;
      }

      const data = await response.json();
      const jwt = data.token || data.jwt;

      if (!jwt) {
        messageDiv.textContent = 'Signup failed: No token received.';
        messageDiv.classList.remove('hidden');
        messageDiv.classList.add('text-red-600');
        return;
      }

      // Store JWT in localStorage
      localStorage.setItem('jwt', jwt);

      localStorage.setItem('username', username);

      messageDiv.textContent = 'Signup successful!';
      messageDiv.classList.remove('hidden');
      messageDiv.classList.remove('text-red-600'); // Remove red if previously set
      messageDiv.classList.add('text-green-600'); // Style for success messages

      // Redirect to homepage or update UI
      setTimeout(() => {
        window.location.href = '/web/index.html';
      }, 1000);

    } catch (err) {
      messageDiv.textContent = 'An error occurred. Please try again.';
      messageDiv.classList.remove('hidden');
      messageDiv.classList.add('text-red-600');
    }
  });
});
