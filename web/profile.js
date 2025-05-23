document.addEventListener('DOMContentLoaded', function () {
    const jwt = localStorage.getItem('jwt');
    const username = localStorage.getItem('username');
    if (!jwt || !username) {
        window.location.href = '/web/login.html';
        return;
    }
    document.getElementById('profile-username').textContent = username;
    document.getElementById('logout-btn').addEventListener('click', () => {
        localStorage.removeItem('jwt');
        localStorage.removeItem('username');
        window.location.href = '/web/index.html';
    });
});