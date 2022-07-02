let $description = document.querySelector("#description")
let descriptionText = $description.textContent;
$description.innerHTML = bbparser(descriptionText)

let $copyButton = document.querySelector("#copy-sha256sum-btn")
let $copyIcon = document.querySelector(".icon-content-copy")
let $doneIcon = document.querySelector(".icon-done")
let $message = document.querySelector("#copy-sha256sum-message")
let sha256sum = document.querySelector("#sha256sum").textContent + "\n"
let messageTimeout

// Clipboard needs localhost or HTTPS
if (navigator.clipboard === undefined) {
  $copyButton.classList.add("hidden")
} else {
  $copyButton.addEventListener("click", e => {
    clearTimeout(messageTimeout)

    navigator.clipboard.writeText(sha256sum)
    $message.classList.remove("hidden")
    $message.classList.add("copy-sha256sum-message-show")
    $copyButton.classList.add("copy-sha256sum-btn-show")
    $copyIcon.classList.add("hidden")
    $doneIcon.classList.remove("hidden")

    messageTimeout = setTimeout(() => {
      $message.classList.add("hidden")
      $message.classList.remove("copy-sha256sum-message-show")
      $copyButton.classList.remove("copy-sha256sum-btn-show")
      $copyIcon.classList.remove("hidden")
      $doneIcon.classList.add("hidden")
    }, 1000)
  })
}

let $selectVersion = document.querySelector("#select-version")
$selectVersion.addEventListener("change", e => {
  let path = e.target.value
  window.location.pathname = path
})

window.addEventListener("pageshow", () => {
  let $selectedVersion = $selectVersion.querySelector("option[selected]")
  $selectVersion.value = $selectedVersion.value
})
