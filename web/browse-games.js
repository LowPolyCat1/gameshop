// src/web/browse-games.js

document.addEventListener('DOMContentLoaded', async () => {
    const gameListingsContainer = document.getElementById('game-listings');
    const loadingIndicator = document.getElementById('loading-indicator');

    // Function to show a message box (reusing the pattern from sell.js)
    function showMessageBox(title, message, isSuccess = true) {
        let messageBox = document.getElementById('messageBox');
        if (!messageBox) {
            messageBox = document.createElement('div');
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

            document.getElementById('messageBoxClose').addEventListener('click', () => {
                messageBox.classList.add('hidden');
            });
        }

        document.getElementById('messageBoxTitle').textContent = title;
        document.getElementById('messageBoxContent').textContent = message;
        if (isSuccess) {
            document.getElementById('messageBoxTitle').classList.remove('text-red-600');
            document.getElementById('messageBoxTitle').classList.add('text-green-600');
        } else {
            document.getElementById('messageBoxTitle').classList.remove('text-green-600');
            document.getElementById('messageBoxTitle').classList.add('text-red-600');
        }
        messageBox.classList.remove('hidden');
    }

    // Function to format date strings
    function formatDateTime(isoString) {
        try {
            const date = new Date(isoString);
            if (isNaN(date.getTime())) {
                throw new Error('Invalid date string');
            }
            return date.toLocaleString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit',
                hour12: true
            });
        } catch (e) {
            console.error('Error formatting date:', e, isoString);
            return isoString; // Return original if formatting fails
        }
    }

    async function fetchAndDisplayOffers() {
        loadingIndicator.classList.remove('hidden');
        gameListingsContainer.innerHTML = ''; // Clear previous listings

        const token = localStorage.getItem('jwt');

        try {
            const response = await fetch('/api/offers', {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    // Include Authorization header if the route is protected
                    ...(token && { 'Authorization': `Bearer ${token}` })
                }
            });

            const result = await response.json();

            if (response.ok) {
                if (result.offers && result.offers.length > 0) {
                    result.offers.forEach(offer => {
                        const offerCard = document.createElement('div');
                        offerCard.className = 'bg-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-shadow duration-300 ease-in-out flex flex-col';

                        // CORRECTED: Access the 'String' property within the 'id' object
                        const sellerIdDisplay = offer.seller_id && offer.seller_id.id && offer.seller_id.id.String
                            ? offer.seller_id.id.String
                            : 'N/A';
                        const formattedPrice = typeof offer.price === 'number' ? `$${offer.price.toFixed(2)}` : 'N/A';
                        const formattedCreatedAt = offer.created_at ? formatDateTime(offer.created_at) : 'N/A';

                        offerCard.innerHTML = `
                            <img src="https://placehold.co/400x250/FFB400/23272F?text=Game+Image" alt="${offer.game_title}" class="w-full h-48 object-cover object-center rounded-t-xl" onerror="this.onerror=null;this.src='https://placehold.co/400x250/FFB400/23272F?text=Image+Error';">
                            <div class="p-6 flex flex-col flex-grow">
                                <h3 class="text-2xl font-bold text-gray-900 mb-2 truncate">${offer.game_title}</h3>
                                <div class="flex flex-col text-left mb-2">
                                    <p class="text-gray-700 font-medium flex items-baseline"><span class="flex-shrink-0 w-20">Platform:</span> <span class="font-normal flex-grow">${offer.platform}</span></p>
                                    <p class="text-gray-700 font-medium flex items-baseline"><span class="flex-shrink-0 w-20">Condition:</span> <span class="font-normal flex-grow">${offer.condition}</span></p>
                                    <p class="text-gray-700 font-medium flex items-baseline"><span class="flex-shrink-0 w-20">Seller:</span> <span class="font-normal text-blue-600 flex-grow">${sellerIdDisplay}</span></p>
                                </div>
                                <p class="text-2xl font-bold text-yellow-600 mb-4 text-left">${formattedPrice}</p>
                                <p class="text-gray-600 text-sm mb-4 line-clamp-3 text-left flex-grow">${offer.description}</p>
                                <div class="text-xs text-gray-500 mt-auto">
                                    <p>Listed: ${formattedCreatedAt}</p>
                                </div>
                                <button class="bg-yellow-500 text-gray-900 font-bold py-2 px-6 rounded-full hover:bg-yellow-600 transition duration-300 ease-in-out shadow-md hover:shadow-lg mt-auto">
                                    View Details
                                </button>
                            </div>
                        `;
                        gameListingsContainer.appendChild(offerCard);
                    });
                } else {
                    gameListingsContainer.innerHTML = `
                        <p class="col-span-full text-center text-gray-600 text-xl py-8">No games listed yet. Be the first to sell one!</p>
                    `;
                }
            } else {
                showMessageBox('Error', result.message || 'Failed to load games. Please try again.', false);
            }
        } catch (error) {
            console.error('Error fetching offers:', error);
            showMessageBox('Network Error', 'Could not connect to the server. Please check your internet connection and try again.', false);
        } finally {
            loadingIndicator.classList.add('hidden');
        }
    }

    // Initial fetch of games when the page loads
    fetchAndDisplayOffers();

    // The infinite scrolling logic will need to be adapted if your backend
    // supports paginated results. For now, this fetches all offers at once.
    // If you implement backend pagination, you'd modify fetchAndDisplayOffers
    // to take page/limit parameters and call it on scroll.
    // window.addEventListener('scroll', handleScroll); // Uncomment and implement if backend pagination is added.
});
