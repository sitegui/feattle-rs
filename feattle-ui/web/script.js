'use strict'

class FeatureEditor {
    constructor(editorEl, outputEl) {
        this.editorEl = editorEl
        this.outputEl = outputEl
        this.sourceEl = null

        this.format = JSON.parse(editorEl.attr('data-format'))
        this.value = JSON.parse(editorEl.attr('data-value'))

        if (this.format.tag == 'Bool') {
            this._bool()
        } else if (this.format.tag == 'Integer') {
            this._number(true)
        } else if (this.format.tag == 'Float') {
            this._number(false)
        } else if (this.format.tag == 'String' && this.format.content.tag == 'Any') {
            this._string()
        } else if (this.format.tag == 'String' && this.format.content.tag == 'Pattern') {
            this._string(this.format.content.content)
        } else if (this.format.tag == 'String' && this.format.content.tag == 'Choices') {
            this._choices(this.format.content.content)
        } else {
            this._other()
        }

        this.editorEl.append(this.sourceEl)
        this.sourceEl.change(() => {
            this._setValue(this.sourceEl.val())
        })
    }

    _bool() {
        this.sourceEl = $('<select>', {
            'class': 'custom-select',
            append: [
                $('<option>', {
                    value: 'true',
                    selected: this.value,
                    text: 'true'
                }),
                $('<option>', {
                    value: 'false',
                    selected: !this.value,
                    text: 'false'
                }),
            ]
        })
    }

    _number(isInteger) {
        this.sourceEl = $('<input>', {
            type: 'number',
            step: isInteger ? '' : 'any',
            val: this.value
        })
    }

    _string(pattern) {
        this.sourceEl = $('<input>', {
            pattern: pattern || '',
            val: this.value
        })
    }

    _choices(choices) {
        this.sourceEl = $('<select>', {
            'class': 'custom-select',
            append: choices.map(choice => $('<option>', {
                value: choice,
                selected: choice == this.value,
                text: choice
            }))
        })
    }

    _setValue(value) {
        console.log('Change value to', JSON.stringify(value))
        this.value = value
        this.outputEl.val(JSON.stringify(value))
    }
}