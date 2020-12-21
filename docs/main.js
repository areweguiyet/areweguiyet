'use strict'

// This file was kind of a rush job ;^)

Vue.component('crate-card', {
    props: ['crate'],
    template: `
        <div class="ecosystem-card">
            <header class="header">
                <h3>{{ crate.name }}</h3>
                <span v-if="meta_links">[ 
                    <template v-for="(link, index) in meta_links">
                        <a v-bind:href="link[1]">{{link[0]}}</a>
                        <template v-if="index < meta_links.length - 1">
                            ·
                        </template>
                    </template>
                 ]</span>
                </template>
            </header>
            <div class="content">
                <p>{{ crate.description }}</p>
            </div>
            <ul v-if="crate.tags" class="ecosystem-tags">
                <li v-for="tag in crate.tags">{{ tag }}</li>
            </ul>
        </div>
    `,
    data: function () {
        let meta_links = []
        if (this.crate.crates_io) {
            meta_links.push(['crate', this.crate.crates_io])
        }
        if (this.crate.repo) {
            meta_links.push(['repo', this.crate.repo])
        }
        if (this.crate.docs) {
            meta_links.push(['docs', this.crate.docs])
        }
        return {
            crate: this.crate,
            meta_links, 
        }
    }
})

Vue.component('crates-list', {
    props: ['crates_map', 'tag_filter'],
    template: `
        <div class="ecosystem-crates">
            <crate-card
                v-show="is_not_filtered_by_tags(crate.tags, tag_filter)"
                v-for="crate in crates"
                v-bind:crate="crate"
                v-bind:key="crate.name">
        </div>
    `,
    data: function () {
        let crates = Object.keys(this.crates_map)
        crates = crates.map(function (key) {
                return {
                    name: key,
                    ...this.crates_map[key]
                }
            }.bind(this))
        crates = crates.sort(function (a, b) {
            return a.name.localeCompare(b.name)
        })
        return {
            crates,
        }
    }
})

// False if crate_tags does not contain anything matching tag filter
function is_not_filtered_by_tags(crate_tags, tag_filter) {
    if (tag_filter.length === 0) {
        // Don't filter if there is no tag filter
        return true;
    }
    for (let crate_tag of crate_tags) {
        if (tag_filter.includes(crate_tag)) {
            return true;
        }
    }
    return false;
}

function load_ecosystem() {
    return fetch('compiled_ecosystem.json')
        .then(function(response) {
            return response.json()
        })
}

function init_crate_list_ui() {
    const tag_filter = []

    // I think there's a better way to do this with Vue.JS, but I'm not that familiar with it
    load_ecosystem()
        .then(function(crates) {
            new Vue({
                el: '#app-crates',
                data: {
                    crates,
                    tag_filter
                }
            })
        })

    // attach event handlers

    // handle filter select/deselect
    document.getElementById('ecosystem-tags').addEventListener('click', function(e) {
        if (e.target.tagName !== 'LI') {
            return
        }
        const tag_name = e.target.getAttribute('data-crate-tag')
        const index = tag_filter.indexOf(tag_name)
        if (index === -1) {
            tag_filter.push(tag_name)
            e.target.className = e.target.className += ' active'
        } else {
            tag_filter.splice(index, 1)
            e.target.className = e.target.className.replace(' active')
        }
    })
}

init_crate_list_ui()
