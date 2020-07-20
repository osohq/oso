window.onload = (evt) => {
    document.querySelectorAll(".gi,.gd").forEach(item => {
        var newItem = item.cloneNode(false);
        newItem.setAttribute("class", item.getAttribute("class") + "n");
        newItem.innerHTML = item.innerHTML.substring(0, 1);
        item.innerHTML = item.innerHTML.substring(1);

        item.insertAdjacentElement("beforebegin", newItem);
    });
}
