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
if (navButton) {
  navButton.addEventListener('click', () => {
    navContent.classList.toggle('hidden');
    navTitle.classList.toggle('hidden');
    navToggleOpen.classList.toggle('hidden');
    navToggleClosed.classList.toggle('hidden');
  });
}

const sideBarContent = document.getElementById('sidebar-content');
const sideBarButton = document.getElementById('sidebar-toggle');
const sideBarSearch = document.getElementById('sidebar-search');
if (sideBarButton) {
  const toggleSideBar = () => sideBarContent.classList.toggle('hidden');
  sideBarButton.addEventListener('click', toggleSideBar);
}

const langButton = document.getElementById('language-selector-toggle');
const langsideBar = document.getElementById('language-selector-content');
if (langButton) {
  langButton.addEventListener('click', () =>
    langsideBar.classList.toggle('hidden')
  );
}

const searchModal = document.getElementById('search-modal');
const searchInput = document.getElementById('search-input');
const searchBackground = document.getElementById('search-background');

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
    case searchInput:
      break
    case searchModal:
      window.hideSearch(event);
      break
    case searchBackground:
      window.hideSearch(event);
      break
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
