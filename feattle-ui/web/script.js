'use strict'

class FeattleEditor {
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
        this.switchEl = this._newSwitch(
            this.editorEl,
            'Define some value',
            this.initialValue !== null
        )

        this.sourceEl = $('<div>', {
            'class': 'py-2',
            attr: {
                'data-format': JSON.stringify(innerFormat),
                'data-value': JSON.stringify(this.initialValue)
            }
        })
        this.editorEl.append(this.sourceEl)
        this.innerEditor = new FeattleEditor(this.sourceEl)

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
        this.getValue = () => {
            let value = JSON.parse(this.innerJSONEditor.getValue())
            this._check(this.format, value)
            return value
        }
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

    _check(format, value) {
        let assert = (test, getMessage) => {
            if (!test) {
                throw new Error(getMessage())
            }
        }

        if (format.tag === 'Bool') {
            assert(typeof value === 'boolean', () => `${value} is not a boolean`)
        } else if (format.tag === 'Integer') {
            assert(Number.isInteger(value), () => `${value} is not an integer`)
        } else if (format.tag === 'Float') {
            assert(Number.isFinite(value), () => `${value} is not a float`)
        } else if (format.tag === 'String') {
            this._checkString(format.content, value)
        } else if (format.tag === 'Optional') {
            if (value !== null) {
                this._check(format.content, value)
            }
        } else if (format.tag === 'List' || format.tag === 'Set') {
            assert(Array.isArray(value), () => `${value} is not an array`)
            value.forEach(el => this._check(format.content, el))
        } else if (format.tag === 'Map') {
            assert(
                value !== null &&
                typeof value === 'object' &&
                !Array.isArray(value), () => `${value} is not an object`
            )
            let keyFormat = format.content[0]
            let elFormat = format.content[1]
            Object.entries(value).forEach(([key, el]) => {
                this._checkString(keyFormat, key)
                this._check(elFormat, el)
            })
        } else {
            assert(false, () => 'Unknown data type')
        }
    }

    _checkString(stringFormat, value) {
        let assert = (test, getMessage) => {
            if (!test) {
                throw new Error(getMessage())
            }
        }

        assert(typeof value === 'string', () => `${value} is not a string`)

        if (stringFormat.tag === 'Any') {
        } else if (stringFormat.tag === 'Pattern') {
            let pattern = new RegExp('^(?:' + stringFormat.content + ')$')
            assert(
                pattern.test(value),
                () => `${value} does not match the pattern ${stringFormat.content}`
            )
        } else if (stringFormat.tag === 'Choices') {
            assert(
                stringFormat.content.includes(value),
                () => `${value} must be one of ${stringFormat.content.join(', ')}`
            )
        } else {
            assert(false, () => 'Unknown data type')
        }
    }
}