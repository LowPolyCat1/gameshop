document.addEventListener('DOMContentLoaded', function () {
    const navbar = document.getElementById('navbar-links');
    // Remove previously injected auth links
    Array.from(navbar.querySelectorAll('.auth-link')).forEach(el => el.remove());

    const jwt = localStorage.getItem('jwt');

    if (jwt) {
        // Remove any hardcoded Sign Up/Login links if present
        Array.from(navbar.querySelectorAll('a')).forEach(link => {
            if (
                link.textContent.trim() === "Sign Up" ||
                link.textContent.trim() === "Login"
            ) {
                link.remove();
            }
        });

        // Profile icon link (no username)
        const profileLink = document.createElement('a');
        profileLink.href = "/web/profile.html";
        profileLink.className = "auth-link profile-link";
        profileLink.style.display = "flex";
        profileLink.style.alignItems = "center";
        profileLink.title = "Profile";
        profileLink.innerHTML = `
            <svg width="22" height="22" viewBox="0 0 24 24" fill="#ffb400" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false">
                <circle cx="12" cy="8" r="4"/>
                <path d="M12 14c-5 0-8 2.5-8 4v2h16v-2c0-1.5-3-4-8-4z"/>
            </svg>
        `;
        navbar.appendChild(profileLink);

        // Logout link
        const logoutLink = document.createElement('a');
        logoutLink.href = "#";
        logoutLink.className = "auth-link";
        logoutLink.textContent = "Logout";
        logoutLink.addEventListener('click', function (e) {
            e.preventDefault();
            localStorage.removeItem('jwt');
            window.location.reload();
        });
        navbar.appendChild(logoutLink);

    } else {
        // Show Sign Up and Login links
        if (!navbar.querySelector('a[href="/web/signup.html"]')) {
            const signupLink = document.createElement('a');
            signupLink.href = "/web/signup.html";
            signupLink.className = "auth-link";
            signupLink.textContent = "Sign Up";
            navbar.appendChild(signupLink);
        }
        if (!navbar.querySelector('a[href="/web/login.html"]')) {
            const loginLink = document.createElement('a');
            loginLink.href = "/web/login.html";
            loginLink.className = "auth-link";
            loginLink.textContent = "Login";
            navbar.appendChild(loginLink);
        }
    }
});
