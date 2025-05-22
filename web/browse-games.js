document.addEventListener('DOMContentLoaded', function () {
    const gameListingsContainer = document.getElementById('game-listings');
    const loadingIndicator = document.getElementById('loading-indicator');

    // Sample game data with images and detailed information
    const gamesData = [
        {
            id: 1,
            title: "Epic Adventure Quest",
            game: "The Legend of Zelda: Breath of the Wild",
            condition: "Used - Excellent",
            seller_username: "LinkFanatic",
            price: "$45.00",
            description: "Immerse yourself in a vast open world! Comes with original case and manual. Barely played, looks new.",
            image: "https://placehold.co/400x250/FFB400/23272F?text=Zelda+BotW"
        },
        {
            id: 2,
            title: "Galactic Strategy Masterpiece",
            game: "Starcraft II: Wings of Liberty",
            condition: "New - Sealed",
            seller_username: "TerranKing",
            price: "$30.00",
            description: "Brand new, still in shrink wrap. Perfect for collectors or new players!",
            image: "https://placehold.co/400x250/23272F/FFFFFF?text=Starcraft+II"
        },
        {
            id: 3,
            title: "Post-Apocalyptic Survival",
            game: "Fallout 4",
            condition: "Used - Good",
            seller_username: "WastelandWanderer",
            price: "$20.00",
            description: "Explore the Commonwealth! Disc has minor scratches but plays perfectly. No manual.",
            image: "https://placehold.co/400x250/B8B8B8/333333?text=Fallout+4"
        },
        {
            id: 4,
            title: "Fantasy RPG Epic",
            game: "The Witcher 3: Wild Hunt (Complete Edition)",
            condition: "Used - Like New",
            seller_username: "GeraltOfRivia",
            price: "$50.00",
            description: "Includes all expansions on disc. Flawless condition, hours of monster hunting fun!",
            image: "https://placehold.co/400x250/6A0DAD/FFFFFF?text=Witcher+3"
        },
        {
            id: 5,
            title: "High-Speed Racing Thrills",
            game: "Forza Horizon 5",
            condition: "Used - Excellent",
            seller_username: "SpeedDemon",
            price: "$40.00",
            description: "Race across Mexico! Disc is pristine. Online code may be used.",
            image: "https://placehold.co/400x250/007BFF/FFFFFF?text=Forza+Horizon+5"
        },
        {
            id: 6,
            title: "Horror Survival Classic",
            game: "Resident Evil Village",
            condition: "Used - Good",
            seller_username: "LadyDimitrescu",
            price: "$35.00",
            description: "Terrifying and immersive! Minor wear on case, disc is fine.",
            image: "https://placehold.co/400x250/8B0000/FFFFFF?text=Resident+Evil"
        },
        {
            id: 7,
            title: "Open World Crime Saga",
            game: "Grand Theft Auto V",
            condition: "Used - Fair",
            seller_username: "LosSantosFinest",
            price: "$15.00",
            description: "Still a blast to play! Disc has some scratches, but works. No map included.",
            image: "https://placehold.co/400x250/FFA500/FFFFFF?text=GTA+V"
        },
        {
            id: 8,
            title: "Cozy Life Simulation",
            game: "Animal Crossing: New Horizons",
            condition: "Used - Excellent",
            seller_username: "IslandDesigner",
            price: "$48.00",
            description: "Build your dream island! Cartridge is perfect, no case.",
            image: "https://placehold.co/400x250/ADD8E6/000000?text=Animal+Crossing"
        },
        {
            id: 9,
            title: "Competitive Shooter",
            game: "Call of Duty: Modern Warfare II",
            condition: "New - Open Box",
            seller_username: "TacticalGamer",
            price: "$55.00",
            description: "Opened but never played. Code unused. Ready for action!",
            image: "https://placehold.co/400x250/4B0082/FFFFFF?text=Call+of+Duty"
        },
        {
            id: 10,
            title: "Fantasy Action-Adventure",
            game: "Elden Ring",
            condition: "Used - Like New",
            seller_username: "TarnishedOne",
            price: "$58.00",
            description: "Challenge the Lands Between! Flawless disc, original case.",
            image: "https://placehold.co/400x250/708090/FFFFFF?text=Elden+Ring"
        },
        {
            id: 11,
            title: "Retro Platformer Fun",
            game: "Super Mario Odyssey",
            condition: "Used - Excellent",
            seller_username: "MushroomKingdom",
            price: "$42.00",
            description: "Capture enemies and explore! Cartridge is perfect.",
            image: "https://placehold.co/400x250/FF6347/FFFFFF?text=Mario+Odyssey"
        },
        {
            id: 12,
            title: "Sci-Fi RPG Saga",
            game: "Mass Effect Legendary Edition",
            condition: "Used - Good",
            seller_username: "CommanderShepard",
            price: "$38.00",
            description: "Relive the epic trilogy! Discs have minor surface scratches, plays fine.",
            image: "https://placehold.co/400x250/4682B4/FFFFFF?text=Mass+Effect"
        },
        {
            id: 13,
            title: "Viking Adventure",
            game: "Assassin's Creed Valhalla",
            condition: "Used - Excellent",
            seller_username: "VikingRaid",
            price: "$30.00",
            description: "Pillage and conquer! Disc is clean, complete with case.",
            image: "https://placehold.co/400x250/006400/FFFFFF?text=AC+Valhalla"
        },
        {
            id: 14,
            title: "Cyberpunk Dystopia",
            game: "Cyberpunk 2077",
            condition: "Used - Good",
            seller_username: "NightCityMerc",
            price: "$25.00",
            description: "Explore Night City! Disc has light wear, plays without issues.",
            image: "https://placehold.co/400x250/FFD700/000000?text=Cyberpunk+2077"
        },
        {
            id: 15,
            title: "Spooky Ghost Hunting",
            game: "Luigi's Mansion 3",
            condition: "Used - Like New",
            seller_username: "PoltergustPro",
            price: "$40.00",
            description: "Ghostly fun for all ages! Perfect condition.",
            image: "https://placehold.co/400x250/8A2BE2/FFFFFF?text=Luigi's+Mansion"
        },
        {
            id: 16,
            title: "Mythological Action",
            game: "God of War Ragnarök",
            condition: "Used - Excellent",
            seller_username: "KratosFan",
            price: "$55.00",
            description: "Journey through the Nine Realms! Disc is immaculate.",
            image: "https://placehold.co/400x250/CD5C5C/FFFFFF?text=GoW+Ragnarok"
        }
    ];

    const gamesPerPage = 8; // Number of games to load at a time
    let currentPage = 0;
    let isLoading = false;

    /**
     * Creates an HTML string for a single game card.
     * @param {object} game - The game object with title, game, condition, seller_username, price, description, image.
     * @returns {string} - HTML string for the game card.
     */
    function createGameCard(game) {
        return `
            <div class="bg-white rounded-xl shadow-lg overflow-hidden transform transition-transform duration-300 hover:scale-105 hover:shadow-xl">
                <img src="${game.image}" alt="${game.game}" class="w-full h-48 object-cover object-center" onerror="this.onerror=null;this.src='https://placehold.co/400x250/FFB400/23272F?text=Image+Error';">
                <div class="p-6">
                    <h3 class="text-xl font-semibold text-gray-900 mb-2 truncate">${game.title}</h3>
                    <p class="text-gray-700 font-medium mb-1">Game: <span class="font-normal">${game.game}</span></p>
                    <p class="text-gray-700 font-medium mb-1">Condition: <span class="font-normal">${game.condition}</span></p>
                    <p class="text-gray-700 font-medium mb-1">Seller: <span class="font-normal text-blue-600">${game.seller_username}</span></p>
                    <p class="text-2xl font-bold text-yellow-600 mb-4">${game.price}</p>
                    <p class="text-gray-600 text-sm mb-4 line-clamp-3">${game.description}</p>
                    <button class="bg-yellow-500 text-gray-900 font-bold py-2 px-6 rounded-full hover:bg-yellow-600 transition duration-300 ease-in-out shadow-md hover:shadow-lg">
                        View Details
                    </button>
                </div>
            </div>
        `;
    }

    /**
     * Loads a batch of games and appends them to the display.
     */
    function loadGames() {
        if (isLoading) return; // Prevent multiple simultaneous loads
        isLoading = true;
        loadingIndicator.classList.remove('hidden'); // Show loading indicator

        const startIndex = currentPage * gamesPerPage;
        const endIndex = startIndex + gamesPerPage;
        const gamesToLoad = gamesData.slice(startIndex, endIndex);

        if (gamesToLoad.length === 0) {
            loadingIndicator.classList.add('hidden'); // Hide if no more games
            return;
        }

        // Simulate a network delay for better UX
        setTimeout(() => {
            gamesToLoad.forEach(game => {
                const gameCardHtml = createGameCard(game);
                gameListingsContainer.insertAdjacentHTML('beforeend', gameCardHtml);
            });

            currentPage++;
            isLoading = false;
            loadingIndicator.classList.add('hidden'); // Hide loading indicator
        }, 500); // 500ms delay
    }

    /**
     * Handles the scroll event to trigger infinite loading.
     */
    function handleScroll() {
        // Check if user has scrolled to the bottom of the page (with a 200px buffer)
        const scrollThreshold = document.documentElement.scrollHeight - window.innerHeight - 200;
        if (window.scrollY >= scrollThreshold && !isLoading && currentPage * gamesPerPage < gamesData.length) {
            loadGames();
        }
    }

    // Initial load of games when the page loads
    loadGames();

    // Add scroll event listener for infinite scrolling
    window.addEventListener('scroll', handleScroll);
});
