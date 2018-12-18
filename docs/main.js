'use strict'

// I started writing this file assuming not all crates would be loaded at once...
//
// To simplify things, we will (for now) just load everything. If this becomes a performance issue 
// we can fix it then! We look forward to that day as it means there's a great ecosystem!

function Cache() {
    this.tag_pool = []
    this.meta_link_pool = []
    // crates card containers not in use
    this.crate_pool = {}
    // crate card containers references keyed by crate name
    this.crates = {}
}

Cache.prototype.toggle_tag_filter = function(tag_name) {
    console.log('You filtered by a tag!')
}

Cache.prototype.set_crates = function(crates) {
    this.crates = crates

    // remove all the old crates from the page
    let crate_html = document.getElementById('ecosystem-crates')
    while (crate_html.lastChild) {
        crate_html.removeChild(crate_html.lastChild)
    }
    
    // add the container to the crates object and make each crate visible on the page
    Object.keys(this.crates).forEach(function(crate_name) {
        let container = _new_card_container()
        _set_card_container(container, crate_name, this.crates[crate_name])
        this.crates[crate_name].container = container

        crate_html.appendChild(container)
    })
}

function _new_card_container() {
    let html = document.createElement('div')

    // create the header block
    html.className = 'ecosystem-card '
    let header = document.createElement('header')
    html.appendChild(header)
    header.className = 'header '

    // create an h3 and text node for the crate name
    header.appendChild(document.createElement('h3'))
    let crate_name_node = document.createTextNode('')

    // create a span for crate meta links (caller is responsible for attribute elements)
    header.lastChild.appendChild(crate_name_node)
    let meta_links_node = document.createElement('span')
    header.appendChild(meta_links_node)

    // create the content block
    let content_block = document.createElement('div')
    html.appendChild(content_block)
    content_block.className = 'content '
    content_block.appendChild(document.createElement('p'))

    let crate_description_node = document.createTextNode('')
    content_block.lastChild.appendChild(crate_description_node)
    
    // create the tags block (caller is responsible for tag elements)
    let tag_list_node = document.createElement('ul')
    html.appendChild(tag_list_node)
    tag_list_node.className = 'ecosystem-tags '

    return {
        crate_name_node,
        meta_links_node,
        crate_description_node,
        tag_list_node,
        html
    }
}

// n.b., this creates attribute and list elements each call
function _set_card_container(card, crate_name, crate_info) {
    card.crate_name_node.nodeValue = crate_name
    card.crate_description_node.nodeValue = crate_info.description
    
    // create the meta links
    let meta_links = []
    if (crate_info.crates_io) {
        meta_links.push(new_meta_link('crate', crate_info.crates_io))
    }
    if (crate_info.repo) {
        meta_links.push(new_meta_link('repo', crate_info.repo))
    }
    if (crate_info.docs) {
        meta_links.push(new_meta_link('docs', crate_info.docs))
    }
    
    // add the meta links, surrounded by [ ], and separated each by a point
    let i
    card.meta_links_node.appendChild(document.createTextNode('['))
    for (i=0; i < meta_links.length; i+=1) {
        if (i !== 0) {
            card.meta_links_node.appendChild(document.createTextNode('·'))
        }
        card.meta_links_node.appendChild(meta_links[i])
    }
    card.meta_links_node.appendChild(document.createTextNode(']'))

    // create the tag list
    for (i=0; i < crate_info.tags.length; i+=1) {
        let element = document.createElement('li')
        card.tag_list_node.appendChild()
        element.appendChild(document.createTextNode(crate_info.tags[i]))
    }
}

function new_meta_link(text, link) {
    let a = document.createElement('a')
    a.setAttribute('href', link)
    a.appendChild(document.createTextNode(text))
    return a
}

function load_ecosystem() {
    return fetch('/ecosystem.json')
        .then(function(response) {
            return response.json()
        })
}

function init_crate_list_ui() {
    let cache = new Cache()

    load_ecosystem()
        .then(function(crates) {
            cache.set_crates(crates)
        })

    // attach event handlers

    // handle filter select/deselect
    document.getElementById('ecosystem-tags').addEventListener('click', function(e) {
        if (e.target.tagName !== 'LI') {
            return
        }
        cache.toggle_tag_filter(e.target.getAttribute('data-crate-tag'))
    })
}
