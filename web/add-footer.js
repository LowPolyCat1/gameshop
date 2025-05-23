// add-footer.js

document.addEventListener('DOMContentLoaded', () => {
    // Check if a footer already exists, to avoid adding duplicates if not desired.
    // If you always want to add a footer regardless, you can remove this check.
    const existingFooter = document.querySelector('footer');

    if (!existingFooter) { // Only add if no footer exists
        // Create the new footer element
        const newFooter = document.createElement('footer');
        // Apply the desired classes for styling
        newFooter.className = 'bg-gray-900 text-white text-center py-4 mt-auto shadow-inner';

        // Create the inner div for the container
        const containerDiv = document.createElement('div');
        containerDiv.className = 'container mx-auto px-4';

        // Create the paragraph element for the copyright text and links
        const paragraph = document.createElement('p');
        paragraph.className = 'text-sm';
        paragraph.innerHTML = '&copy; ' + new Date().getFullYear() + ' GameSwap. Peer-to-peer videogame marketplace. ' +
            '<a href="/web/impressum.html" class="text-blue-200 hover:underline">Impressum</a> | ' +
            '<a href="/web/privacy.html" class="text-blue-200 hover:underline">Privacy Policy</a>';

        // Append the paragraph to the container div
        containerDiv.appendChild(paragraph);

        // Append the container div to the new footer
        newFooter.appendChild(containerDiv);

        // Append the new footer to the body of the document
        document.body.appendChild(newFooter);

        console.log('Footer added successfully!');
    } else {
        console.log('An existing footer was found, no new footer added by add-footer.js.');
    }
});
