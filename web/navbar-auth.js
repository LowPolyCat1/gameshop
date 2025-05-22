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
        // Tailwind classes for navigation links
        a.className = 'text-white hover:text-yellow-500 transition duration-200 ease-in-out px-3 py-2 rounded-md';
        navbar.appendChild(a);
    });

    // 2) Auth‐specific links
    const jwt = localStorage.getItem('jwt');

    if (jwt) {
        // Profile icon link
        const profileLink = document.createElement('a');
        profileLink.href = '/web/profile.html';
        // Tailwind classes for profile link button
        profileLink.className = 'ml-4 px-3 py-2 rounded-md flex items-center justify-center bg-yellow-500 hover:bg-yellow-600 transition duration-200 ease-in-out';
        profileLink.title = 'Profile';
        profileLink.innerHTML = `
          <svg class="w-6 h-6 text-gray-900" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false">
            <circle cx="12" cy="8" r="4"/>
            <path d="M12 14c-5 0-8 2.5-8 4v2h16v-2c0-1.5-3-4-8-4z"/>
          </svg>
        `;
        navbar.appendChild(profileLink);

        // Logout link
        const logoutLink = document.createElement('a');
        logoutLink.href = '#';
        // Tailwind classes for logout link
        logoutLink.className = 'text-white hover:text-yellow-500 transition duration-200 ease-in-out px-3 py-2 rounded-md';
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
        // Tailwind classes for signup link
        signupLink.className = 'text-white hover:text-yellow-500 transition duration-200 ease-in-out px-3 py-2 rounded-md';
        navbar.appendChild(signupLink);

        // Login
        const loginLink = document.createElement('a');
        loginLink.href = '/web/login.html';
        // Tailwind classes for login link
        loginLink.className = 'text-white hover:text-yellow-500 transition duration-200 ease-in-out px-3 py-2 rounded-md';
        navbar.appendChild(loginLink);
    }
});
