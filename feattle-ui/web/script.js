'use strict'

class FeatureEditor {
    constructor(editorEl, outputEl) {
        this.editorEl = editorEl
        this.outputEl = outputEl
        this.sourceEl = null

        this.format = editorEl.data('format')
        this.value = editorEl.data('value')

        if (this.format.tag == 'Bool') {
            this._checkbox()
        } else if (this.format.tag == 'Number') {
        } else if (this.format.tag == 'String') {
        }
    }

    _checkbox() {
        this.sourceEl = $('<input>', {
            'type': 'checkbox',
            'checked': this.value
        })
        this.editorEl.append(this.sourceEl)
        this.sourceEl.change(() => {
            this._setValue(this.sourceEl.prop('checked'))
        })
    }

    _setValue(value) {
        console.log('Change value to', value)
        this.value = value
        this.outputEl = JSON.stringify(value)
    }
}