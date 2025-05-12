document.addEventListener('DOMContentLoaded', function () {
    const navbar = document.getElementById('navbar-links');

    // Clear out any existing links (optional—useful if you reload)
    navbar.innerHTML = '';

    // 1) Core navigation links
    const coreLinks = [
        { href: '/web/index.html', text: 'Home' },
        { href: '/web/browse.html', text: 'Browse Games' },
        { href: '/web/sell.html', text: 'Sell a Game' },
    ];
    coreLinks.forEach(({ href, text }) => {
        const a = document.createElement('a');
        a.href = href;
        a.textContent = text;
        navbar.appendChild(a);
    });

    // Separator (optional)
    // const sep = document.createElement('span');
    // sep.textContent = ' | ';
    // navbar.appendChild(sep);

    // 2) Auth‐specific links
    const jwt = localStorage.getItem('jwt');

    if (jwt) {
        // Profile icon link
        const profileLink = document.createElement('a');
        profileLink.href = '/web/profile.html';
        profileLink.className = 'auth-link profile-link';
        profileLink.title = 'Profile';
        profileLink.innerHTML = `
          <svg width="22" height="22" viewBox="0 0 24 24" fill="#ffb400" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false">
            <circle cx="12" cy="8" r="4"/>
            <path d="M12 14c-5 0-8 2.5-8 4v2h16v-2c0-1.5-3-4-8-4z"/>
          </svg>
        `;
        navbar.appendChild(profileLink);

        // Logout link
        const logoutLink = document.createElement('a');
        logoutLink.href = '#';
        logoutLink.className = 'auth-link';
        logoutLink.textContent = 'Logout';
        logoutLink.addEventListener('click', function (e) {
            e.preventDefault();
            localStorage.removeItem('jwt');
            localStorage.removeItem('username');
            window.location.reload();
        });
        navbar.appendChild(logoutLink);

    } else {
        // Sign Up
        const signupLink = document.createElement('a');
        signupLink.href = '/web/signup.html';
        signupLink.className = 'auth-link';
        signupLink.textContent = 'Sign Up';
        navbar.appendChild(signupLink);

        // Login
        const loginLink = document.createElement('a');
        loginLink.href = '/web/login.html';
        loginLink.className = 'auth-link';
        loginLink.textContent = 'Login';
        navbar.appendChild(loginLink);
    }
});
