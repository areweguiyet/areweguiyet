'use strict'

function setTagFilter(tag) {
    const button = document.querySelector(`#ecosystem-tags li[data-ecosystem-tag="${tag}"]`)
    button.classList.add("active")
    updateCrates()
}

function toggleTagFilter(tag) {
    const button = document.querySelector(`#ecosystem-tags li[data-ecosystem-tag="${tag}"]`)
    button.classList.toggle("active")
    updateCrates()
}

// Update the `hidden` CSS class on crates that should be filtered
function updateCrates() {
    const buttons = document.querySelectorAll(`#ecosystem-tags li`)
    const filteredTags = []
    for (let button of buttons) {
        if (button.classList.contains("active")) {
            filteredTags.push(button.dataset.ecosystemTag)
        }
    }

    const crates = document.querySelectorAll(`.ecosystem-crates .ecosystem-card`)
    for (let crate of crates) {
        const tags = crate.dataset.tags.split(',').map(tag => tag.trim())
        if (isFilteredByTags(tags, filteredTags)) {
            crate.classList.add("hidden")
        } else {
            crate.classList.remove("hidden")
        }
    }
}

// Whether a crate should be filtered according to the given filter
function isFilteredByTags(tags, filteredTags) {
    if (filteredTags.length === 0) {
        // Don't filter if there is no tag filter
        return false;
    }

    for (let tag of tags) {
        if (filteredTags.includes(tag)) {
            return false;
        }
    }

    return true;
}
