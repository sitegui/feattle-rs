'use strict'

class FeatureEditor {
    constructor(editorEl) {
        this.editorEl = editorEl

        this.format = JSON.parse(editorEl.attr('data-format'))
        this.initialValue = JSON.parse(editorEl.attr('data-value'))

        this.sourceEl = null
        this.getValue = null

        if (this.format.tag === 'Bool') {
            this._prepareBool()
        } else if (this.format.tag === 'Integer') {
            this._prepareNumber(true)
        } else if (this.format.tag === 'Float') {
            this._prepareNumber(false)
        } else if (this.format.tag === 'String' && this.format.content.tag === 'Any') {
            this._prepareString()
        } else if (this.format.tag === 'String' && this.format.content.tag === 'Pattern') {
            this._prepareString(this.format.content.content)
        } else if (this.format.tag === 'String' && this.format.content.tag === 'Choices') {
            this._prepareChoices(this.format.content.content)
        } else if (this.format.tag === 'Optional') {
            this._prepareOptional(this.format.content)
        } else {
            this._prepareOther()
        }
    }

    _prepareBool() {
        this.sourceEl = this._newSwitch(this.editorEl, 'Value', this.initialValue)
        this.getValue = () => this.sourceEl.prop('checked')
    }

    _prepareNumber(isInteger) {
        this.sourceEl = $('<input>', {
            'class': 'form-control',
            type: 'number',
            step: isInteger ? '' : 'any',
            val: this.initialValue
        })
        this.editorEl.append(this.sourceEl)
        this.getValue = () => Number(this.sourceEl.val())
    }

    _prepareString(pattern) {
        this.sourceEl = $('<input>', {
            'class': 'form-control',
            pattern: pattern,
            val: this.initialValue
        })
        this.editorEl.append(this.sourceEl)
        this.getValue = () => this.sourceEl.val()
    }

    _prepareChoices(choices) {
        let radioGroupId = String(Math.random())
        this.sourceEl = $(choices.map(choice => {
            let radioId = String(Math.random())
            return $('<div>', {
                'class': 'custom-control custom-radio',
                append: [
                    $('<input>', {
                        type: 'radio',
                        name: radioGroupId,
                        id: radioId,
                        'class': 'custom-control-input',
                        value: choice,
                        checked: choice === this.initialValue
                    }),
                    $('<label>', {
                        'class': 'custom-control-label',
                        'for': radioId,
                        text: choice
                    })
                ]
            }).get(0)
        }))
        this.editorEl.append(this.sourceEl)
        this.getValue = () => $('input:checked', this.sourceEl).val()
    }

    _prepareOptional(innerFormat) {
        this.switchEl = this._newSwitch(this.editorEl, 'Define some value', this.initialValue !== null)

        this.sourceEl = $('<div>', {
            'class': 'py-2',
            attr: {
                'data-format': JSON.stringify(innerFormat),
                'data-value': JSON.stringify(this.initialValue)
            }
        })
        this.editorEl.append(this.sourceEl)
        this.innerEditor = new FeatureEditor(this.sourceEl)

        this.switchEl.change(() => {
            if (this.switchEl.prop('checked')) {
                this.sourceEl.show()
            } else {
                this.sourceEl.hide()
            }
        })
        if (this.initialValue === null) {
            this.sourceEl.hide()
        }

        this.getValue = () => this.switchEl.prop('checked') ? this.innerEditor.getValue() : null
    }

    _prepareOther() {
        this.sourceEl = $('<div>', {
            style: 'height: 10em; font-size: 1em',
            text: JSON.stringify(this.initialValue, null, 4)
        })
        this.editorEl.append(this.sourceEl)
        this.innerJSONEditor = ace.edit(this.sourceEl.get(0))
        this.innerJSONEditor.setTheme('ace/theme/monokai')
        this.innerJSONEditor.session.setMode('ace/mode/json')
        this.getValue = () => JSON.parse(this.innerJSONEditor.getValue())
    }

    _newSwitch(rootEl, label, initialValue) {
        let inputId = String(Math.random())
        let switchEl = $('<input>', {
            type: 'checkbox',
            'class': 'custom-control-input',
            id: inputId,
            checked: initialValue
        })
        rootEl.append($('<div>', {
            'class': 'custom-control custom-switch',
            append: [
                switchEl,
                $('<label>', {
                    'class': 'custom-control-label',
                    text: label,
                    'for': inputId
                })
            ]
        }))
        return switchEl
    }
}