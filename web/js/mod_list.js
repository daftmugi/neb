let $mods = document.querySelectorAll(".mod-item")
let $searchBox = document.getElementById("search-box")

let searchInputTimeout
let searchInterval = 150
$searchBox.addEventListener("keydown", () => {
  clearTimeout(searchInputTimeout)
  searchInputTimeout = setTimeout(search, searchInterval)
})

document.addEventListener("keyup", (e) => {
  if (e.code == "KeyS") {
    $searchBox.focus()
  }
})

function search() {
  requestAnimationFrame(() => {
    let query = $searchBox.value.toLowerCase()

    for (let $mod of $mods) {
      let text = $mod.textContent.toLowerCase()

      if (text.includes(query)) {
        $mod.classList.remove("is-hidden")
      } else {
        $mod.classList.add("is-hidden")
      }
    }
  })
}
