const monaco = import('monaco-editor');

// Basic navigation functionality
document.querySelector('button > svg').addEventListener('click', function(e) {
  e.stopPropagation();
  e.preventDefault();
});

const navContent = document.getElementById('nav-content');
const navButton = document.getElementById('nav-toggle');
const navToggleOpen = document.getElementById('nav-toggle-open');
const navToggleClosed = document.getElementById('nav-toggle-closed');
const navTitle = document.getElementById('header-page-title');
navButton.addEventListener('click', () => {
  navContent.classList.toggle('hidden');
  navTitle.classList.toggle('hidden');
  navToggleOpen.classList.toggle('hidden');
  navToggleClosed.classList.toggle('hidden');
});

const sideBarContent = document.getElementById('sidebar-content');
const sideBarButton = document.getElementById('sidebar-toggle');
const sideBarSearch = document.getElementById('sidebar-search');
if (sideBarButton) {
  const toggleSideBar = () => sideBarContent.classList.toggle('hidden');
  sideBarButton.addEventListener('click', toggleSideBar);
}

const langButton = document.getElementById('language-selector-toggle');
const langsideBar = document.getElementById('language-selector-content');
langButton.addEventListener('click', () =>
  langsideBar.classList.toggle('hidden')
);

// Close dropdown sideBars if the user clicks outside of them
window.onclick = function(event) {
  console.log(event.target);
  switch (event.target) {
    case navButton:
      break;
    case sideBarButton:
      break;
    case sideBarSearch:
      break;
    case langButton:
      break;

    default:
      // default to hidden
      var contents = [navContent, langsideBar, sideBarContent, navToggleOpen];

      for (content of contents) {
        if (content && !content.classList.contains('hidden')) {
          content.classList.toggle('hidden');
        }
      }

      // default to visible
      var contents = [navTitle, navToggleClosed];

      for (content of contents) {
        if (content && content.classList.contains('hidden')) {
          content.classList.toggle('hidden');
        }
      }
      break;
  }
};

//Get the button:
mybutton = document.getElementById('scroll-to-top');

// When the user scrolls down 20px from the top of the document, show the button
window.onscroll = function() {
  scrollFunction();
};

function scrollFunction() {
  if (document.body.scrollTop > 20 || document.documentElement.scrollTop > 20) {
    mybutton.style.display = 'block';
  } else {
    mybutton.style.display = 'none';
  }
}

// When the user clicks on the button, scroll to the top of the document
function topFunction() {
  document.body.scrollTop = 0; // For Safari
  document.documentElement.scrollTop = 0; // For Chrome, Firefox, IE and Opera
}
