// src/web/sell.js

document.addEventListener('DOMContentLoaded', () => {
    const form = document.querySelector('form');
    const titleInput = document.getElementById('title');
    const platformInput = document.getElementById('platform');
    const conditionSelect = document.getElementById('condition');
    const priceInput = document.getElementById('price');
    const descriptionTextarea = document.getElementById('description');

    // Message box elements
    const messageBox = document.createElement('div');
    messageBox.id = 'messageBox';
    messageBox.className = 'fixed inset-0 bg-gray-900 bg-opacity-50 flex items-center justify-center z-50 hidden';
    messageBox.innerHTML = `
        <div class="bg-white p-6 rounded-lg shadow-xl max-w-sm w-full text-center relative">
            <h3 id="messageBoxTitle" class="text-2xl font-bold mb-4"></h3>
            <p id="messageBoxContent" class="text-gray-700 mb-6"></p>
            <button id="messageBoxClose" class="bg-yellow-500 text-gray-900 font-bold py-2 px-4 rounded-full hover:bg-yellow-600 transition duration-300 ease-in-out">
                OK
            </button>
        </div>
    `;
    document.body.appendChild(messageBox);

    const messageBoxTitle = document.getElementById('messageBoxTitle');
    const messageBoxContent = document.getElementById('messageBoxContent');
    const messageBoxCloseButton = document.getElementById('messageBoxClose');

    messageBoxCloseButton.addEventListener('click', () => {
        messageBox.classList.add('hidden');
    });

    function showMessageBox(title, message, isSuccess = true) {
        messageBoxTitle.textContent = title;
        messageBoxContent.textContent = message;
        if (isSuccess) {
            messageBoxTitle.classList.remove('text-red-600');
            messageBoxTitle.classList.add('text-green-600');
        } else {
            messageBoxTitle.classList.remove('text-green-600');
            messageBoxTitle.classList.add('text-red-600');
        }
        messageBox.classList.remove('hidden');
    }

    form.addEventListener('submit', async (event) => {
        event.preventDefault(); // Prevent default form submission

        // Client-side validation (basic, backend will do full validation)
        const gameTitle = titleInput.value.trim();
        const platform = platformInput.value.trim();
        const condition = conditionSelect.value;
        const price = parseFloat(priceInput.value);
        const description = descriptionTextarea.value.trim();

        if (!gameTitle || !platform || !condition || isNaN(price) || price <= 0 || !description) {
            showMessageBox('Validation Error', 'Please fill in all fields correctly. Price must be a positive number.', false);
            return;
        }

        if (gameTitle.length < 3) {
            showMessageBox('Validation Error', 'Game title must be at least 3 characters long.', false);
            return;
        }
        if (platform.length < 2) {
            showMessageBox('Validation Error', 'Platform must be at least 2 characters long.', false);
            return;
        }
        if (condition.length < 2) {
            showMessageBox('Validation Error', 'Condition must be at least 2 characters long.', false);
            return;
        }
        if (description.length < 10) {
            showMessageBox('Validation Error', 'Description must be at least 10 characters long.', false);
            return;
        }

        const token = localStorage.getItem('jwt');

        if (!token) {
            showMessageBox('Authentication Error', 'You must be logged in to list a game.', false);
            // Redirect to login page if no token
            setTimeout(() => {
                window.location.href = '/web/login.html';
            }, 2000);
            return;
        }

        const offerData = {
            game_title: gameTitle,
            platform: platform,
            condition: condition,
            price: price,
            description: description,
        };

        try {
            const response = await fetch('/api/offers', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify(offerData)
            });

            const result = await response.json();

            if (response.ok) {
                showMessageBox('Success!', result.message || 'Game listed successfully!', true);
                // Clear the form after successful submission
                form.reset();
            } else {
                showMessageBox('Error', result.message || 'Failed to list game. Please try again.', false);
            }
        } catch (error) {
            console.error('Error listing game:', error);
            showMessageBox('Network Error', 'Could not connect to the server. Please check your internet connection and try again.', false);
        }
    });
});
