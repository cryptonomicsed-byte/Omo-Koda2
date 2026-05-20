/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars, default-case*/
"use strict";

var $protobuf = require("protobufjs/minimal");

// Common aliases
var $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
var $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

$root.omokoda = (function() {

    /**
     * Namespace omokoda.
     * @exports omokoda
     * @namespace
     */
    var omokoda = {};

    omokoda.v1 = (function() {

        /**
         * Namespace v1.
         * @memberof omokoda
         * @namespace
         */
        var v1 = {};

        v1.AgentBorn = (function() {

            /**
             * Properties of an AgentBorn.
             * @typedef {Object} omokoda.v1.AgentBorn.$Properties
             * @property {string|null} [dna] AgentBorn dna
             * @property {Array.<string>|null} [mnemonic] AgentBorn mnemonic
             * @property {number|null} [odu] AgentBorn odu
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of an AgentBorn.
             * @memberof omokoda.v1
             * @interface IAgentBorn
             * @augments omokoda.v1.AgentBorn.$Properties
             * @deprecated Use omokoda.v1.AgentBorn.$Properties instead.
             */

            /**
             * Shape of an AgentBorn.
             * @typedef {omokoda.v1.AgentBorn.$Properties} omokoda.v1.AgentBorn.$Shape
             */

            /**
             * Constructs a new AgentBorn.
             * @memberof omokoda.v1
             * @classdesc Represents an AgentBorn.
             * @constructor
             * @param {omokoda.v1.AgentBorn.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function AgentBorn(properties) {
                this.mnemonic = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * AgentBorn dna.
             * @member {string} dna
             * @memberof omokoda.v1.AgentBorn
             * @instance
             */
            AgentBorn.prototype.dna = "";

            /**
             * AgentBorn mnemonic.
             * @member {Array.<string>} mnemonic
             * @memberof omokoda.v1.AgentBorn
             * @instance
             */
            AgentBorn.prototype.mnemonic = $util.emptyArray;

            /**
             * AgentBorn odu.
             * @member {number} odu
             * @memberof omokoda.v1.AgentBorn
             * @instance
             */
            AgentBorn.prototype.odu = 0;

            /**
             * Creates a new AgentBorn instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {omokoda.v1.AgentBorn.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.AgentBorn} AgentBorn instance
             * @type {{
             *   (properties: omokoda.v1.AgentBorn.$Shape): omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape;
             *   (properties?: omokoda.v1.AgentBorn.$Properties): omokoda.v1.AgentBorn;
             * }}
             */
            AgentBorn.create = function create(properties) {
                return new AgentBorn(properties);
            };

            /**
             * Encodes the specified AgentBorn message. Does not implicitly {@link omokoda.v1.AgentBorn.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {omokoda.v1.AgentBorn.$Properties} message AgentBorn message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AgentBorn.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.dna != null && Object.hasOwnProperty.call(message, "dna"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.dna);
                if (message.mnemonic != null && message.mnemonic.length)
                    for (var i = 0; i < message.mnemonic.length; ++i)
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.mnemonic[i]);
                if (message.odu != null && Object.hasOwnProperty.call(message, "odu"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.odu);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified AgentBorn message, length delimited. Does not implicitly {@link omokoda.v1.AgentBorn.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {omokoda.v1.AgentBorn.$Properties} message AgentBorn message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AgentBorn.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AgentBorn message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape} AgentBorn
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AgentBorn.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.AgentBorn(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.string()).length)
                                message.dna = value;
                            else
                                delete message.dna;
                            continue;
                        }
                    case 2: {
                            if (wireType !== 2)
                                break;
                            if (!(message.mnemonic && message.mnemonic.length))
                                message.mnemonic = [];
                            message.mnemonic.push(reader.string());
                            continue;
                        }
                    case 3: {
                            if (wireType !== 0)
                                break;
                            if (value = reader.uint32())
                                message.odu = value;
                            else
                                delete message.odu;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes an AgentBorn message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape} AgentBorn
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AgentBorn.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AgentBorn message.
             * @function verify
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AgentBorn.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.dna != null && message.hasOwnProperty("dna"))
                    if (!$util.isString(message.dna))
                        return "dna: string expected";
                if (message.mnemonic != null && message.hasOwnProperty("mnemonic")) {
                    if (!Array.isArray(message.mnemonic))
                        return "mnemonic: array expected";
                    for (var i = 0; i < message.mnemonic.length; ++i)
                        if (!$util.isString(message.mnemonic[i]))
                            return "mnemonic: string[] expected";
                }
                if (message.odu != null && message.hasOwnProperty("odu"))
                    if (!$util.isInteger(message.odu))
                        return "odu: integer expected";
                return null;
            };

            /**
             * Creates an AgentBorn message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.AgentBorn} AgentBorn
             */
            AgentBorn.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.AgentBorn)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.AgentBorn();
                if (object.dna != null)
                    if (typeof object.dna !== "string" || object.dna.length)
                        message.dna = String(object.dna);
                if (object.mnemonic) {
                    if (!Array.isArray(object.mnemonic))
                        throw TypeError(".omokoda.v1.AgentBorn.mnemonic: array expected");
                    message.mnemonic = Array(object.mnemonic.length);
                    for (var i = 0; i < object.mnemonic.length; ++i)
                        message.mnemonic[i] = String(object.mnemonic[i]);
                }
                if (object.odu != null)
                    if (Number(object.odu) !== 0)
                        message.odu = object.odu >>> 0;
                return message;
            };

            /**
             * Creates a plain object from an AgentBorn message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {omokoda.v1.AgentBorn} message AgentBorn
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AgentBorn.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.mnemonic = [];
                if (options.defaults) {
                    object.dna = "";
                    object.odu = 0;
                }
                if (message.dna != null && message.hasOwnProperty("dna"))
                    object.dna = message.dna;
                if (message.mnemonic && message.mnemonic.length) {
                    object.mnemonic = Array(message.mnemonic.length);
                    for (var j = 0; j < message.mnemonic.length; ++j)
                        object.mnemonic[j] = message.mnemonic[j];
                }
                if (message.odu != null && message.hasOwnProperty("odu"))
                    object.odu = message.odu;
                return object;
            };

            /**
             * Converts this AgentBorn to JSON.
             * @function toJSON
             * @memberof omokoda.v1.AgentBorn
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AgentBorn.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for AgentBorn
             * @function getTypeUrl
             * @memberof omokoda.v1.AgentBorn
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            AgentBorn.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.AgentBorn";
            };

            return AgentBorn;
        })();

        v1.ThoughtSealed = (function() {

            /**
             * Properties of a ThoughtSealed.
             * @typedef {Object} omokoda.v1.ThoughtSealed.$Properties
             * @property {Uint8Array|null} [intentHash] ThoughtSealed intentHash
             * @property {number|null} [hermeticScore] ThoughtSealed hermeticScore
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of a ThoughtSealed.
             * @memberof omokoda.v1
             * @interface IThoughtSealed
             * @augments omokoda.v1.ThoughtSealed.$Properties
             * @deprecated Use omokoda.v1.ThoughtSealed.$Properties instead.
             */

            /**
             * Shape of a ThoughtSealed.
             * @typedef {omokoda.v1.ThoughtSealed.$Properties} omokoda.v1.ThoughtSealed.$Shape
             */

            /**
             * Constructs a new ThoughtSealed.
             * @memberof omokoda.v1
             * @classdesc Represents a ThoughtSealed.
             * @constructor
             * @param {omokoda.v1.ThoughtSealed.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function ThoughtSealed(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ThoughtSealed intentHash.
             * @member {Uint8Array} intentHash
             * @memberof omokoda.v1.ThoughtSealed
             * @instance
             */
            ThoughtSealed.prototype.intentHash = $util.newBuffer([]);

            /**
             * ThoughtSealed hermeticScore.
             * @member {number} hermeticScore
             * @memberof omokoda.v1.ThoughtSealed
             * @instance
             */
            ThoughtSealed.prototype.hermeticScore = 0;

            /**
             * Creates a new ThoughtSealed instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {omokoda.v1.ThoughtSealed.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.ThoughtSealed} ThoughtSealed instance
             * @type {{
             *   (properties: omokoda.v1.ThoughtSealed.$Shape): omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape;
             *   (properties?: omokoda.v1.ThoughtSealed.$Properties): omokoda.v1.ThoughtSealed;
             * }}
             */
            ThoughtSealed.create = function create(properties) {
                return new ThoughtSealed(properties);
            };

            /**
             * Encodes the specified ThoughtSealed message. Does not implicitly {@link omokoda.v1.ThoughtSealed.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {omokoda.v1.ThoughtSealed.$Properties} message ThoughtSealed message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ThoughtSealed.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.intentHash != null && Object.hasOwnProperty.call(message, "intentHash"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.intentHash);
                if (message.hermeticScore != null && Object.hasOwnProperty.call(message, "hermeticScore"))
                    writer.uint32(/* id 2, wireType 5 =*/21).float(message.hermeticScore);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified ThoughtSealed message, length delimited. Does not implicitly {@link omokoda.v1.ThoughtSealed.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {omokoda.v1.ThoughtSealed.$Properties} message ThoughtSealed message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ThoughtSealed.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ThoughtSealed message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape} ThoughtSealed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ThoughtSealed.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.ThoughtSealed(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.bytes()).length)
                                message.intentHash = value;
                            else
                                delete message.intentHash;
                            continue;
                        }
                    case 2: {
                            if (wireType !== 5)
                                break;
                            if ((value = reader.float()) !== 0)
                                message.hermeticScore = value;
                            else
                                delete message.hermeticScore;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes a ThoughtSealed message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape} ThoughtSealed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ThoughtSealed.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ThoughtSealed message.
             * @function verify
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ThoughtSealed.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.intentHash != null && message.hasOwnProperty("intentHash"))
                    if (!(message.intentHash && typeof message.intentHash.length === "number" || $util.isString(message.intentHash)))
                        return "intentHash: buffer expected";
                if (message.hermeticScore != null && message.hasOwnProperty("hermeticScore"))
                    if (typeof message.hermeticScore !== "number")
                        return "hermeticScore: number expected";
                return null;
            };

            /**
             * Creates a ThoughtSealed message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.ThoughtSealed} ThoughtSealed
             */
            ThoughtSealed.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.ThoughtSealed)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.ThoughtSealed();
                if (object.intentHash != null)
                    if (object.intentHash.length)
                        if (typeof object.intentHash === "string")
                            $util.base64.decode(object.intentHash, message.intentHash = $util.newBuffer($util.base64.length(object.intentHash)), 0);
                        else if (object.intentHash.length >= 0)
                            message.intentHash = object.intentHash;
                if (object.hermeticScore != null)
                    if (Number(object.hermeticScore) !== 0)
                        message.hermeticScore = Number(object.hermeticScore);
                return message;
            };

            /**
             * Creates a plain object from a ThoughtSealed message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {omokoda.v1.ThoughtSealed} message ThoughtSealed
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ThoughtSealed.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.intentHash = "";
                    else {
                        object.intentHash = [];
                        if (options.bytes !== Array)
                            object.intentHash = $util.newBuffer(object.intentHash);
                    }
                    object.hermeticScore = 0;
                }
                if (message.intentHash != null && message.hasOwnProperty("intentHash"))
                    object.intentHash = options.bytes === String ? $util.base64.encode(message.intentHash, 0, message.intentHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.intentHash) : message.intentHash;
                if (message.hermeticScore != null && message.hasOwnProperty("hermeticScore"))
                    object.hermeticScore = options.json && !isFinite(message.hermeticScore) ? String(message.hermeticScore) : message.hermeticScore;
                return object;
            };

            /**
             * Converts this ThoughtSealed to JSON.
             * @function toJSON
             * @memberof omokoda.v1.ThoughtSealed
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ThoughtSealed.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for ThoughtSealed
             * @function getTypeUrl
             * @memberof omokoda.v1.ThoughtSealed
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            ThoughtSealed.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.ThoughtSealed";
            };

            return ThoughtSealed;
        })();

        v1.ActExecuted = (function() {

            /**
             * Properties of an ActExecuted.
             * @typedef {Object} omokoda.v1.ActExecuted.$Properties
             * @property {string|null} [tool] ActExecuted tool
             * @property {Uint8Array|null} [receiptMerkle] ActExecuted receiptMerkle
             * @property {number|null} [f1Score] ActExecuted f1Score
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of an ActExecuted.
             * @memberof omokoda.v1
             * @interface IActExecuted
             * @augments omokoda.v1.ActExecuted.$Properties
             * @deprecated Use omokoda.v1.ActExecuted.$Properties instead.
             */

            /**
             * Shape of an ActExecuted.
             * @typedef {omokoda.v1.ActExecuted.$Properties} omokoda.v1.ActExecuted.$Shape
             */

            /**
             * Constructs a new ActExecuted.
             * @memberof omokoda.v1
             * @classdesc Represents an ActExecuted.
             * @constructor
             * @param {omokoda.v1.ActExecuted.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function ActExecuted(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ActExecuted tool.
             * @member {string} tool
             * @memberof omokoda.v1.ActExecuted
             * @instance
             */
            ActExecuted.prototype.tool = "";

            /**
             * ActExecuted receiptMerkle.
             * @member {Uint8Array} receiptMerkle
             * @memberof omokoda.v1.ActExecuted
             * @instance
             */
            ActExecuted.prototype.receiptMerkle = $util.newBuffer([]);

            /**
             * ActExecuted f1Score.
             * @member {number} f1Score
             * @memberof omokoda.v1.ActExecuted
             * @instance
             */
            ActExecuted.prototype.f1Score = 0;

            /**
             * Creates a new ActExecuted instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {omokoda.v1.ActExecuted.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.ActExecuted} ActExecuted instance
             * @type {{
             *   (properties: omokoda.v1.ActExecuted.$Shape): omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape;
             *   (properties?: omokoda.v1.ActExecuted.$Properties): omokoda.v1.ActExecuted;
             * }}
             */
            ActExecuted.create = function create(properties) {
                return new ActExecuted(properties);
            };

            /**
             * Encodes the specified ActExecuted message. Does not implicitly {@link omokoda.v1.ActExecuted.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {omokoda.v1.ActExecuted.$Properties} message ActExecuted message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ActExecuted.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.tool != null && Object.hasOwnProperty.call(message, "tool"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.tool);
                if (message.receiptMerkle != null && Object.hasOwnProperty.call(message, "receiptMerkle"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.receiptMerkle);
                if (message.f1Score != null && Object.hasOwnProperty.call(message, "f1Score"))
                    writer.uint32(/* id 3, wireType 5 =*/29).float(message.f1Score);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified ActExecuted message, length delimited. Does not implicitly {@link omokoda.v1.ActExecuted.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {omokoda.v1.ActExecuted.$Properties} message ActExecuted message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ActExecuted.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an ActExecuted message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape} ActExecuted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ActExecuted.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.ActExecuted(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.string()).length)
                                message.tool = value;
                            else
                                delete message.tool;
                            continue;
                        }
                    case 2: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.bytes()).length)
                                message.receiptMerkle = value;
                            else
                                delete message.receiptMerkle;
                            continue;
                        }
                    case 3: {
                            if (wireType !== 5)
                                break;
                            if ((value = reader.float()) !== 0)
                                message.f1Score = value;
                            else
                                delete message.f1Score;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes an ActExecuted message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape} ActExecuted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ActExecuted.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an ActExecuted message.
             * @function verify
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ActExecuted.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.tool != null && message.hasOwnProperty("tool"))
                    if (!$util.isString(message.tool))
                        return "tool: string expected";
                if (message.receiptMerkle != null && message.hasOwnProperty("receiptMerkle"))
                    if (!(message.receiptMerkle && typeof message.receiptMerkle.length === "number" || $util.isString(message.receiptMerkle)))
                        return "receiptMerkle: buffer expected";
                if (message.f1Score != null && message.hasOwnProperty("f1Score"))
                    if (typeof message.f1Score !== "number")
                        return "f1Score: number expected";
                return null;
            };

            /**
             * Creates an ActExecuted message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.ActExecuted} ActExecuted
             */
            ActExecuted.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.ActExecuted)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.ActExecuted();
                if (object.tool != null)
                    if (typeof object.tool !== "string" || object.tool.length)
                        message.tool = String(object.tool);
                if (object.receiptMerkle != null)
                    if (object.receiptMerkle.length)
                        if (typeof object.receiptMerkle === "string")
                            $util.base64.decode(object.receiptMerkle, message.receiptMerkle = $util.newBuffer($util.base64.length(object.receiptMerkle)), 0);
                        else if (object.receiptMerkle.length >= 0)
                            message.receiptMerkle = object.receiptMerkle;
                if (object.f1Score != null)
                    if (Number(object.f1Score) !== 0)
                        message.f1Score = Number(object.f1Score);
                return message;
            };

            /**
             * Creates a plain object from an ActExecuted message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {omokoda.v1.ActExecuted} message ActExecuted
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ActExecuted.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.tool = "";
                    if (options.bytes === String)
                        object.receiptMerkle = "";
                    else {
                        object.receiptMerkle = [];
                        if (options.bytes !== Array)
                            object.receiptMerkle = $util.newBuffer(object.receiptMerkle);
                    }
                    object.f1Score = 0;
                }
                if (message.tool != null && message.hasOwnProperty("tool"))
                    object.tool = message.tool;
                if (message.receiptMerkle != null && message.hasOwnProperty("receiptMerkle"))
                    object.receiptMerkle = options.bytes === String ? $util.base64.encode(message.receiptMerkle, 0, message.receiptMerkle.length) : options.bytes === Array ? Array.prototype.slice.call(message.receiptMerkle) : message.receiptMerkle;
                if (message.f1Score != null && message.hasOwnProperty("f1Score"))
                    object.f1Score = options.json && !isFinite(message.f1Score) ? String(message.f1Score) : message.f1Score;
                return object;
            };

            /**
             * Converts this ActExecuted to JSON.
             * @function toJSON
             * @memberof omokoda.v1.ActExecuted
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ActExecuted.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for ActExecuted
             * @function getTypeUrl
             * @memberof omokoda.v1.ActExecuted
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            ActExecuted.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.ActExecuted";
            };

            return ActExecuted;
        })();

        v1.TocMinted = (function() {

            /**
             * Properties of a TocMinted.
             * @typedef {Object} omokoda.v1.TocMinted.$Properties
             * @property {string|null} [agent] TocMinted agent
             * @property {number|Long|null} [dopamineBurned] TocMinted dopamineBurned
             * @property {number|Long|null} [synapseEarned] TocMinted synapseEarned
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of a TocMinted.
             * @memberof omokoda.v1
             * @interface ITocMinted
             * @augments omokoda.v1.TocMinted.$Properties
             * @deprecated Use omokoda.v1.TocMinted.$Properties instead.
             */

            /**
             * Shape of a TocMinted.
             * @typedef {omokoda.v1.TocMinted.$Properties} omokoda.v1.TocMinted.$Shape
             */

            /**
             * Constructs a new TocMinted.
             * @memberof omokoda.v1
             * @classdesc Represents a TocMinted.
             * @constructor
             * @param {omokoda.v1.TocMinted.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function TocMinted(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TocMinted agent.
             * @member {string} agent
             * @memberof omokoda.v1.TocMinted
             * @instance
             */
            TocMinted.prototype.agent = "";

            /**
             * TocMinted dopamineBurned.
             * @member {number|Long} dopamineBurned
             * @memberof omokoda.v1.TocMinted
             * @instance
             */
            TocMinted.prototype.dopamineBurned = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * TocMinted synapseEarned.
             * @member {number|Long} synapseEarned
             * @memberof omokoda.v1.TocMinted
             * @instance
             */
            TocMinted.prototype.synapseEarned = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new TocMinted instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {omokoda.v1.TocMinted.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.TocMinted} TocMinted instance
             * @type {{
             *   (properties: omokoda.v1.TocMinted.$Shape): omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape;
             *   (properties?: omokoda.v1.TocMinted.$Properties): omokoda.v1.TocMinted;
             * }}
             */
            TocMinted.create = function create(properties) {
                return new TocMinted(properties);
            };

            /**
             * Encodes the specified TocMinted message. Does not implicitly {@link omokoda.v1.TocMinted.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {omokoda.v1.TocMinted.$Properties} message TocMinted message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TocMinted.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.agent != null && Object.hasOwnProperty.call(message, "agent"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.agent);
                if (message.dopamineBurned != null && Object.hasOwnProperty.call(message, "dopamineBurned"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.dopamineBurned);
                if (message.synapseEarned != null && Object.hasOwnProperty.call(message, "synapseEarned"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.synapseEarned);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified TocMinted message, length delimited. Does not implicitly {@link omokoda.v1.TocMinted.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {omokoda.v1.TocMinted.$Properties} message TocMinted message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TocMinted.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TocMinted message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape} TocMinted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TocMinted.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.TocMinted(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.string()).length)
                                message.agent = value;
                            else
                                delete message.agent;
                            continue;
                        }
                    case 2: {
                            if (wireType !== 0)
                                break;
                            if (typeof (value = reader.uint64()) === "object" ? value.low || value.high : value !== 0)
                                message.dopamineBurned = value;
                            else
                                delete message.dopamineBurned;
                            continue;
                        }
                    case 3: {
                            if (wireType !== 0)
                                break;
                            if (typeof (value = reader.uint64()) === "object" ? value.low || value.high : value !== 0)
                                message.synapseEarned = value;
                            else
                                delete message.synapseEarned;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes a TocMinted message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape} TocMinted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TocMinted.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TocMinted message.
             * @function verify
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TocMinted.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.agent != null && message.hasOwnProperty("agent"))
                    if (!$util.isString(message.agent))
                        return "agent: string expected";
                if (message.dopamineBurned != null && message.hasOwnProperty("dopamineBurned"))
                    if (!$util.isInteger(message.dopamineBurned) && !(message.dopamineBurned && $util.isInteger(message.dopamineBurned.low) && $util.isInteger(message.dopamineBurned.high)))
                        return "dopamineBurned: integer|Long expected";
                if (message.synapseEarned != null && message.hasOwnProperty("synapseEarned"))
                    if (!$util.isInteger(message.synapseEarned) && !(message.synapseEarned && $util.isInteger(message.synapseEarned.low) && $util.isInteger(message.synapseEarned.high)))
                        return "synapseEarned: integer|Long expected";
                return null;
            };

            /**
             * Creates a TocMinted message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.TocMinted} TocMinted
             */
            TocMinted.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.TocMinted)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.TocMinted();
                if (object.agent != null)
                    if (typeof object.agent !== "string" || object.agent.length)
                        message.agent = String(object.agent);
                if (object.dopamineBurned != null)
                    if (typeof object.dopamineBurned === "object" ? object.dopamineBurned.low || object.dopamineBurned.high : Number(object.dopamineBurned) !== 0)
                        if ($util.Long)
                            (message.dopamineBurned = $util.Long.fromValue(object.dopamineBurned)).unsigned = true;
                        else if (typeof object.dopamineBurned === "string")
                            message.dopamineBurned = parseInt(object.dopamineBurned, 10);
                        else if (typeof object.dopamineBurned === "number")
                            message.dopamineBurned = object.dopamineBurned;
                        else if (typeof object.dopamineBurned === "object")
                            message.dopamineBurned = new $util.LongBits(object.dopamineBurned.low >>> 0, object.dopamineBurned.high >>> 0).toNumber(true);
                if (object.synapseEarned != null)
                    if (typeof object.synapseEarned === "object" ? object.synapseEarned.low || object.synapseEarned.high : Number(object.synapseEarned) !== 0)
                        if ($util.Long)
                            (message.synapseEarned = $util.Long.fromValue(object.synapseEarned)).unsigned = true;
                        else if (typeof object.synapseEarned === "string")
                            message.synapseEarned = parseInt(object.synapseEarned, 10);
                        else if (typeof object.synapseEarned === "number")
                            message.synapseEarned = object.synapseEarned;
                        else if (typeof object.synapseEarned === "object")
                            message.synapseEarned = new $util.LongBits(object.synapseEarned.low >>> 0, object.synapseEarned.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from a TocMinted message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {omokoda.v1.TocMinted} message TocMinted
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TocMinted.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.agent = "";
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.dopamineBurned = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.dopamineBurned = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.synapseEarned = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : typeof BigInt !== "undefined" && options.longs === BigInt ? long.toBigInt() : long;
                    } else
                        object.synapseEarned = options.longs === String ? "0" : typeof BigInt !== "undefined" && options.longs === BigInt ? BigInt("0") : 0;
                }
                if (message.agent != null && message.hasOwnProperty("agent"))
                    object.agent = message.agent;
                if (message.dopamineBurned != null && message.hasOwnProperty("dopamineBurned"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.dopamineBurned = typeof message.dopamineBurned === "number" ? BigInt(message.dopamineBurned) : $util.Long.fromBits(message.dopamineBurned.low >>> 0, message.dopamineBurned.high >>> 0, true).toBigInt();
                    else if (typeof message.dopamineBurned === "number")
                        object.dopamineBurned = options.longs === String ? String(message.dopamineBurned) : message.dopamineBurned;
                    else
                        object.dopamineBurned = options.longs === String ? $util.Long.prototype.toString.call(message.dopamineBurned) : options.longs === Number ? new $util.LongBits(message.dopamineBurned.low >>> 0, message.dopamineBurned.high >>> 0).toNumber(true) : message.dopamineBurned;
                if (message.synapseEarned != null && message.hasOwnProperty("synapseEarned"))
                    if (typeof BigInt !== "undefined" && options.longs === BigInt)
                        object.synapseEarned = typeof message.synapseEarned === "number" ? BigInt(message.synapseEarned) : $util.Long.fromBits(message.synapseEarned.low >>> 0, message.synapseEarned.high >>> 0, true).toBigInt();
                    else if (typeof message.synapseEarned === "number")
                        object.synapseEarned = options.longs === String ? String(message.synapseEarned) : message.synapseEarned;
                    else
                        object.synapseEarned = options.longs === String ? $util.Long.prototype.toString.call(message.synapseEarned) : options.longs === Number ? new $util.LongBits(message.synapseEarned.low >>> 0, message.synapseEarned.high >>> 0).toNumber(true) : message.synapseEarned;
                return object;
            };

            /**
             * Converts this TocMinted to JSON.
             * @function toJSON
             * @memberof omokoda.v1.TocMinted
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TocMinted.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for TocMinted
             * @function getTypeUrl
             * @memberof omokoda.v1.TocMinted
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            TocMinted.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.TocMinted";
            };

            return TocMinted;
        })();

        v1.TierAdvanced = (function() {

            /**
             * Properties of a TierAdvanced.
             * @typedef {Object} omokoda.v1.TierAdvanced.$Properties
             * @property {string|null} [agent] TierAdvanced agent
             * @property {number|null} [oldTier] TierAdvanced oldTier
             * @property {number|null} [newTier] TierAdvanced newTier
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of a TierAdvanced.
             * @memberof omokoda.v1
             * @interface ITierAdvanced
             * @augments omokoda.v1.TierAdvanced.$Properties
             * @deprecated Use omokoda.v1.TierAdvanced.$Properties instead.
             */

            /**
             * Shape of a TierAdvanced.
             * @typedef {omokoda.v1.TierAdvanced.$Properties} omokoda.v1.TierAdvanced.$Shape
             */

            /**
             * Constructs a new TierAdvanced.
             * @memberof omokoda.v1
             * @classdesc Represents a TierAdvanced.
             * @constructor
             * @param {omokoda.v1.TierAdvanced.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function TierAdvanced(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TierAdvanced agent.
             * @member {string} agent
             * @memberof omokoda.v1.TierAdvanced
             * @instance
             */
            TierAdvanced.prototype.agent = "";

            /**
             * TierAdvanced oldTier.
             * @member {number} oldTier
             * @memberof omokoda.v1.TierAdvanced
             * @instance
             */
            TierAdvanced.prototype.oldTier = 0;

            /**
             * TierAdvanced newTier.
             * @member {number} newTier
             * @memberof omokoda.v1.TierAdvanced
             * @instance
             */
            TierAdvanced.prototype.newTier = 0;

            /**
             * Creates a new TierAdvanced instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {omokoda.v1.TierAdvanced.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.TierAdvanced} TierAdvanced instance
             * @type {{
             *   (properties: omokoda.v1.TierAdvanced.$Shape): omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape;
             *   (properties?: omokoda.v1.TierAdvanced.$Properties): omokoda.v1.TierAdvanced;
             * }}
             */
            TierAdvanced.create = function create(properties) {
                return new TierAdvanced(properties);
            };

            /**
             * Encodes the specified TierAdvanced message. Does not implicitly {@link omokoda.v1.TierAdvanced.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {omokoda.v1.TierAdvanced.$Properties} message TierAdvanced message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TierAdvanced.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.agent != null && Object.hasOwnProperty.call(message, "agent"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.agent);
                if (message.oldTier != null && Object.hasOwnProperty.call(message, "oldTier"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.oldTier);
                if (message.newTier != null && Object.hasOwnProperty.call(message, "newTier"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.newTier);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified TierAdvanced message, length delimited. Does not implicitly {@link omokoda.v1.TierAdvanced.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {omokoda.v1.TierAdvanced.$Properties} message TierAdvanced message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TierAdvanced.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TierAdvanced message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape} TierAdvanced
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TierAdvanced.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.TierAdvanced(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.string()).length)
                                message.agent = value;
                            else
                                delete message.agent;
                            continue;
                        }
                    case 2: {
                            if (wireType !== 0)
                                break;
                            if (value = reader.uint32())
                                message.oldTier = value;
                            else
                                delete message.oldTier;
                            continue;
                        }
                    case 3: {
                            if (wireType !== 0)
                                break;
                            if (value = reader.uint32())
                                message.newTier = value;
                            else
                                delete message.newTier;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes a TierAdvanced message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape} TierAdvanced
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TierAdvanced.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TierAdvanced message.
             * @function verify
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TierAdvanced.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.agent != null && message.hasOwnProperty("agent"))
                    if (!$util.isString(message.agent))
                        return "agent: string expected";
                if (message.oldTier != null && message.hasOwnProperty("oldTier"))
                    if (!$util.isInteger(message.oldTier))
                        return "oldTier: integer expected";
                if (message.newTier != null && message.hasOwnProperty("newTier"))
                    if (!$util.isInteger(message.newTier))
                        return "newTier: integer expected";
                return null;
            };

            /**
             * Creates a TierAdvanced message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.TierAdvanced} TierAdvanced
             */
            TierAdvanced.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.TierAdvanced)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.TierAdvanced();
                if (object.agent != null)
                    if (typeof object.agent !== "string" || object.agent.length)
                        message.agent = String(object.agent);
                if (object.oldTier != null)
                    if (Number(object.oldTier) !== 0)
                        message.oldTier = object.oldTier >>> 0;
                if (object.newTier != null)
                    if (Number(object.newTier) !== 0)
                        message.newTier = object.newTier >>> 0;
                return message;
            };

            /**
             * Creates a plain object from a TierAdvanced message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {omokoda.v1.TierAdvanced} message TierAdvanced
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TierAdvanced.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.agent = "";
                    object.oldTier = 0;
                    object.newTier = 0;
                }
                if (message.agent != null && message.hasOwnProperty("agent"))
                    object.agent = message.agent;
                if (message.oldTier != null && message.hasOwnProperty("oldTier"))
                    object.oldTier = message.oldTier;
                if (message.newTier != null && message.hasOwnProperty("newTier"))
                    object.newTier = message.newTier;
                return object;
            };

            /**
             * Converts this TierAdvanced to JSON.
             * @function toJSON
             * @memberof omokoda.v1.TierAdvanced
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TierAdvanced.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for TierAdvanced
             * @function getTypeUrl
             * @memberof omokoda.v1.TierAdvanced
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            TierAdvanced.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.TierAdvanced";
            };

            return TierAdvanced;
        })();

        v1.AuditPassed = (function() {

            /**
             * Properties of an AuditPassed.
             * @typedef {Object} omokoda.v1.AuditPassed.$Properties
             * @property {string|null} [receiptId] AuditPassed receiptId
             * @property {Uint8Array|null} [zangbetoSig] AuditPassed zangbetoSig
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of an AuditPassed.
             * @memberof omokoda.v1
             * @interface IAuditPassed
             * @augments omokoda.v1.AuditPassed.$Properties
             * @deprecated Use omokoda.v1.AuditPassed.$Properties instead.
             */

            /**
             * Shape of an AuditPassed.
             * @typedef {omokoda.v1.AuditPassed.$Properties} omokoda.v1.AuditPassed.$Shape
             */

            /**
             * Constructs a new AuditPassed.
             * @memberof omokoda.v1
             * @classdesc Represents an AuditPassed.
             * @constructor
             * @param {omokoda.v1.AuditPassed.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function AuditPassed(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * AuditPassed receiptId.
             * @member {string} receiptId
             * @memberof omokoda.v1.AuditPassed
             * @instance
             */
            AuditPassed.prototype.receiptId = "";

            /**
             * AuditPassed zangbetoSig.
             * @member {Uint8Array} zangbetoSig
             * @memberof omokoda.v1.AuditPassed
             * @instance
             */
            AuditPassed.prototype.zangbetoSig = $util.newBuffer([]);

            /**
             * Creates a new AuditPassed instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {omokoda.v1.AuditPassed.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.AuditPassed} AuditPassed instance
             * @type {{
             *   (properties: omokoda.v1.AuditPassed.$Shape): omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape;
             *   (properties?: omokoda.v1.AuditPassed.$Properties): omokoda.v1.AuditPassed;
             * }}
             */
            AuditPassed.create = function create(properties) {
                return new AuditPassed(properties);
            };

            /**
             * Encodes the specified AuditPassed message. Does not implicitly {@link omokoda.v1.AuditPassed.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {omokoda.v1.AuditPassed.$Properties} message AuditPassed message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AuditPassed.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.receiptId != null && Object.hasOwnProperty.call(message, "receiptId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.receiptId);
                if (message.zangbetoSig != null && Object.hasOwnProperty.call(message, "zangbetoSig"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.zangbetoSig);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified AuditPassed message, length delimited. Does not implicitly {@link omokoda.v1.AuditPassed.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {omokoda.v1.AuditPassed.$Properties} message AuditPassed message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AuditPassed.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AuditPassed message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape} AuditPassed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AuditPassed.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.AuditPassed(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.string()).length)
                                message.receiptId = value;
                            else
                                delete message.receiptId;
                            continue;
                        }
                    case 2: {
                            if (wireType !== 2)
                                break;
                            if ((value = reader.bytes()).length)
                                message.zangbetoSig = value;
                            else
                                delete message.zangbetoSig;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes an AuditPassed message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape} AuditPassed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AuditPassed.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AuditPassed message.
             * @function verify
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AuditPassed.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.receiptId != null && message.hasOwnProperty("receiptId"))
                    if (!$util.isString(message.receiptId))
                        return "receiptId: string expected";
                if (message.zangbetoSig != null && message.hasOwnProperty("zangbetoSig"))
                    if (!(message.zangbetoSig && typeof message.zangbetoSig.length === "number" || $util.isString(message.zangbetoSig)))
                        return "zangbetoSig: buffer expected";
                return null;
            };

            /**
             * Creates an AuditPassed message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.AuditPassed} AuditPassed
             */
            AuditPassed.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.AuditPassed)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.AuditPassed();
                if (object.receiptId != null)
                    if (typeof object.receiptId !== "string" || object.receiptId.length)
                        message.receiptId = String(object.receiptId);
                if (object.zangbetoSig != null)
                    if (object.zangbetoSig.length)
                        if (typeof object.zangbetoSig === "string")
                            $util.base64.decode(object.zangbetoSig, message.zangbetoSig = $util.newBuffer($util.base64.length(object.zangbetoSig)), 0);
                        else if (object.zangbetoSig.length >= 0)
                            message.zangbetoSig = object.zangbetoSig;
                return message;
            };

            /**
             * Creates a plain object from an AuditPassed message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {omokoda.v1.AuditPassed} message AuditPassed
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AuditPassed.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.receiptId = "";
                    if (options.bytes === String)
                        object.zangbetoSig = "";
                    else {
                        object.zangbetoSig = [];
                        if (options.bytes !== Array)
                            object.zangbetoSig = $util.newBuffer(object.zangbetoSig);
                    }
                }
                if (message.receiptId != null && message.hasOwnProperty("receiptId"))
                    object.receiptId = message.receiptId;
                if (message.zangbetoSig != null && message.hasOwnProperty("zangbetoSig"))
                    object.zangbetoSig = options.bytes === String ? $util.base64.encode(message.zangbetoSig, 0, message.zangbetoSig.length) : options.bytes === Array ? Array.prototype.slice.call(message.zangbetoSig) : message.zangbetoSig;
                return object;
            };

            /**
             * Converts this AuditPassed to JSON.
             * @function toJSON
             * @memberof omokoda.v1.AuditPassed
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AuditPassed.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for AuditPassed
             * @function getTypeUrl
             * @memberof omokoda.v1.AuditPassed
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            AuditPassed.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.AuditPassed";
            };

            return AuditPassed;
        })();

        v1.SabbathEntered = (function() {

            /**
             * Properties of a SabbathEntered.
             * @typedef {Object} omokoda.v1.SabbathEntered.$Properties
             * @property {Array.<string>|null} [agentsPaused] SabbathEntered agentsPaused
             * @property {number|null} [queuedOps] SabbathEntered queuedOps
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of a SabbathEntered.
             * @memberof omokoda.v1
             * @interface ISabbathEntered
             * @augments omokoda.v1.SabbathEntered.$Properties
             * @deprecated Use omokoda.v1.SabbathEntered.$Properties instead.
             */

            /**
             * Shape of a SabbathEntered.
             * @typedef {omokoda.v1.SabbathEntered.$Properties} omokoda.v1.SabbathEntered.$Shape
             */

            /**
             * Constructs a new SabbathEntered.
             * @memberof omokoda.v1
             * @classdesc Represents a SabbathEntered.
             * @constructor
             * @param {omokoda.v1.SabbathEntered.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function SabbathEntered(properties) {
                this.agentsPaused = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * SabbathEntered agentsPaused.
             * @member {Array.<string>} agentsPaused
             * @memberof omokoda.v1.SabbathEntered
             * @instance
             */
            SabbathEntered.prototype.agentsPaused = $util.emptyArray;

            /**
             * SabbathEntered queuedOps.
             * @member {number} queuedOps
             * @memberof omokoda.v1.SabbathEntered
             * @instance
             */
            SabbathEntered.prototype.queuedOps = 0;

            /**
             * Creates a new SabbathEntered instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {omokoda.v1.SabbathEntered.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.SabbathEntered} SabbathEntered instance
             * @type {{
             *   (properties: omokoda.v1.SabbathEntered.$Shape): omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape;
             *   (properties?: omokoda.v1.SabbathEntered.$Properties): omokoda.v1.SabbathEntered;
             * }}
             */
            SabbathEntered.create = function create(properties) {
                return new SabbathEntered(properties);
            };

            /**
             * Encodes the specified SabbathEntered message. Does not implicitly {@link omokoda.v1.SabbathEntered.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {omokoda.v1.SabbathEntered.$Properties} message SabbathEntered message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SabbathEntered.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.agentsPaused != null && message.agentsPaused.length)
                    for (var i = 0; i < message.agentsPaused.length; ++i)
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.agentsPaused[i]);
                if (message.queuedOps != null && Object.hasOwnProperty.call(message, "queuedOps"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.queuedOps);
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified SabbathEntered message, length delimited. Does not implicitly {@link omokoda.v1.SabbathEntered.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {omokoda.v1.SabbathEntered.$Properties} message SabbathEntered message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SabbathEntered.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a SabbathEntered message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape} SabbathEntered
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SabbathEntered.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.SabbathEntered(), value;
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            if (!(message.agentsPaused && message.agentsPaused.length))
                                message.agentsPaused = [];
                            message.agentsPaused.push(reader.string());
                            continue;
                        }
                    case 2: {
                            if (wireType !== 0)
                                break;
                            if (value = reader.uint32())
                                message.queuedOps = value;
                            else
                                delete message.queuedOps;
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes a SabbathEntered message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape} SabbathEntered
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SabbathEntered.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a SabbathEntered message.
             * @function verify
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            SabbathEntered.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                if (message.agentsPaused != null && message.hasOwnProperty("agentsPaused")) {
                    if (!Array.isArray(message.agentsPaused))
                        return "agentsPaused: array expected";
                    for (var i = 0; i < message.agentsPaused.length; ++i)
                        if (!$util.isString(message.agentsPaused[i]))
                            return "agentsPaused: string[] expected";
                }
                if (message.queuedOps != null && message.hasOwnProperty("queuedOps"))
                    if (!$util.isInteger(message.queuedOps))
                        return "queuedOps: integer expected";
                return null;
            };

            /**
             * Creates a SabbathEntered message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.SabbathEntered} SabbathEntered
             */
            SabbathEntered.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.SabbathEntered)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.SabbathEntered();
                if (object.agentsPaused) {
                    if (!Array.isArray(object.agentsPaused))
                        throw TypeError(".omokoda.v1.SabbathEntered.agentsPaused: array expected");
                    message.agentsPaused = Array(object.agentsPaused.length);
                    for (var i = 0; i < object.agentsPaused.length; ++i)
                        message.agentsPaused[i] = String(object.agentsPaused[i]);
                }
                if (object.queuedOps != null)
                    if (Number(object.queuedOps) !== 0)
                        message.queuedOps = object.queuedOps >>> 0;
                return message;
            };

            /**
             * Creates a plain object from a SabbathEntered message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {omokoda.v1.SabbathEntered} message SabbathEntered
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            SabbathEntered.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.agentsPaused = [];
                if (options.defaults)
                    object.queuedOps = 0;
                if (message.agentsPaused && message.agentsPaused.length) {
                    object.agentsPaused = Array(message.agentsPaused.length);
                    for (var j = 0; j < message.agentsPaused.length; ++j)
                        object.agentsPaused[j] = message.agentsPaused[j];
                }
                if (message.queuedOps != null && message.hasOwnProperty("queuedOps"))
                    object.queuedOps = message.queuedOps;
                return object;
            };

            /**
             * Converts this SabbathEntered to JSON.
             * @function toJSON
             * @memberof omokoda.v1.SabbathEntered
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            SabbathEntered.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for SabbathEntered
             * @function getTypeUrl
             * @memberof omokoda.v1.SabbathEntered
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            SabbathEntered.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.SabbathEntered";
            };

            return SabbathEntered;
        })();

        v1.SovereignEvent = (function() {

            /**
             * Properties of a SovereignEvent.
             * @typedef {Object} omokoda.v1.SovereignEvent.$Properties
             * @property {omokoda.v1.AgentBorn.$Properties|null} [agentBorn] SovereignEvent agentBorn
             * @property {omokoda.v1.ThoughtSealed.$Properties|null} [thoughtSealed] SovereignEvent thoughtSealed
             * @property {omokoda.v1.ActExecuted.$Properties|null} [actExecuted] SovereignEvent actExecuted
             * @property {omokoda.v1.TocMinted.$Properties|null} [tocMinted] SovereignEvent tocMinted
             * @property {omokoda.v1.TierAdvanced.$Properties|null} [tierAdvanced] SovereignEvent tierAdvanced
             * @property {omokoda.v1.AuditPassed.$Properties|null} [auditPassed] SovereignEvent auditPassed
             * @property {omokoda.v1.SabbathEntered.$Properties|null} [sabbathEntered] SovereignEvent sabbathEntered
             * @property {"agentBorn"|"thoughtSealed"|"actExecuted"|"tocMinted"|"tierAdvanced"|"auditPassed"|"sabbathEntered"} [event] SovereignEvent event
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */

            /**
             * Properties of a SovereignEvent.
             * @memberof omokoda.v1
             * @interface ISovereignEvent
             * @augments omokoda.v1.SovereignEvent.$Properties
             * @deprecated Use omokoda.v1.SovereignEvent.$Properties instead.
             */

            /**
             * Narrowed shape of a SovereignEvent.
             * @typedef {{
             *   agentBorn?: omokoda.v1.AgentBorn.$Shape|null;
             *   thoughtSealed?: omokoda.v1.ThoughtSealed.$Shape|null;
             *   actExecuted?: omokoda.v1.ActExecuted.$Shape|null;
             *   tocMinted?: omokoda.v1.TocMinted.$Shape|null;
             *   tierAdvanced?: omokoda.v1.TierAdvanced.$Shape|null;
             *   auditPassed?: omokoda.v1.AuditPassed.$Shape|null;
             *   sabbathEntered?: omokoda.v1.SabbathEntered.$Shape|null;
             *   $unknowns?: Array.<Uint8Array>;
             * } & (
             *   ({ event?: undefined; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "agentBorn"; agentBorn: omokoda.v1.AgentBorn.$Shape; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "thoughtSealed"; agentBorn?: null; thoughtSealed: omokoda.v1.ThoughtSealed.$Shape; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "actExecuted"; agentBorn?: null; thoughtSealed?: null; actExecuted: omokoda.v1.ActExecuted.$Shape; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "tocMinted"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted: omokoda.v1.TocMinted.$Shape; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "tierAdvanced"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced: omokoda.v1.TierAdvanced.$Shape; auditPassed?: null; sabbathEntered?: null }|{ event?: "auditPassed"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed: omokoda.v1.AuditPassed.$Shape; sabbathEntered?: null }|{ event?: "sabbathEntered"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered: omokoda.v1.SabbathEntered.$Shape })
             * )} omokoda.v1.SovereignEvent.$Shape
             */

            /**
             * Constructs a new SovereignEvent.
             * @memberof omokoda.v1
             * @classdesc Represents a SovereignEvent.
             * @constructor
             * @param {omokoda.v1.SovereignEvent.$Properties=} [properties] Properties to set
             * @property {Array.<Uint8Array>} [$unknowns] Unknown fields preserved while decoding
             */
            function SovereignEvent(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null && keys[i] !== "__proto__")
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * SovereignEvent agentBorn.
             * @member {omokoda.v1.AgentBorn.$Properties|null|undefined} agentBorn
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.agentBorn = null;

            /**
             * SovereignEvent thoughtSealed.
             * @member {omokoda.v1.ThoughtSealed.$Properties|null|undefined} thoughtSealed
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.thoughtSealed = null;

            /**
             * SovereignEvent actExecuted.
             * @member {omokoda.v1.ActExecuted.$Properties|null|undefined} actExecuted
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.actExecuted = null;

            /**
             * SovereignEvent tocMinted.
             * @member {omokoda.v1.TocMinted.$Properties|null|undefined} tocMinted
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.tocMinted = null;

            /**
             * SovereignEvent tierAdvanced.
             * @member {omokoda.v1.TierAdvanced.$Properties|null|undefined} tierAdvanced
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.tierAdvanced = null;

            /**
             * SovereignEvent auditPassed.
             * @member {omokoda.v1.AuditPassed.$Properties|null|undefined} auditPassed
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.auditPassed = null;

            /**
             * SovereignEvent sabbathEntered.
             * @member {omokoda.v1.SabbathEntered.$Properties|null|undefined} sabbathEntered
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            SovereignEvent.prototype.sabbathEntered = null;

            // OneOf field names bound to virtual getters and setters
            var $oneOfFields;

            /**
             * SovereignEvent event.
             * @member {"agentBorn"|"thoughtSealed"|"actExecuted"|"tocMinted"|"tierAdvanced"|"auditPassed"|"sabbathEntered"|undefined} event
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             */
            Object.defineProperty(SovereignEvent.prototype, "event", {
                get: $util.oneOfGetter($oneOfFields = ["agentBorn", "thoughtSealed", "actExecuted", "tocMinted", "tierAdvanced", "auditPassed", "sabbathEntered"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new SovereignEvent instance using the specified properties.
             * @function create
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {omokoda.v1.SovereignEvent.$Properties=} [properties] Properties to set
             * @returns {omokoda.v1.SovereignEvent} SovereignEvent instance
             * @type {{
             *   (properties: omokoda.v1.SovereignEvent.$Shape): omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape;
             *   (properties?: omokoda.v1.SovereignEvent.$Properties): omokoda.v1.SovereignEvent;
             * }}
             */
            SovereignEvent.create = function create(properties) {
                return new SovereignEvent(properties);
            };

            /**
             * Encodes the specified SovereignEvent message. Does not implicitly {@link omokoda.v1.SovereignEvent.verify|verify} messages.
             * @function encode
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {omokoda.v1.SovereignEvent.$Properties} message SovereignEvent message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SovereignEvent.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.agentBorn != null && Object.hasOwnProperty.call(message, "agentBorn"))
                    $root.omokoda.v1.AgentBorn.encode(message.agentBorn, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.thoughtSealed != null && Object.hasOwnProperty.call(message, "thoughtSealed"))
                    $root.omokoda.v1.ThoughtSealed.encode(message.thoughtSealed, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.actExecuted != null && Object.hasOwnProperty.call(message, "actExecuted"))
                    $root.omokoda.v1.ActExecuted.encode(message.actExecuted, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.tocMinted != null && Object.hasOwnProperty.call(message, "tocMinted"))
                    $root.omokoda.v1.TocMinted.encode(message.tocMinted, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.tierAdvanced != null && Object.hasOwnProperty.call(message, "tierAdvanced"))
                    $root.omokoda.v1.TierAdvanced.encode(message.tierAdvanced, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.auditPassed != null && Object.hasOwnProperty.call(message, "auditPassed"))
                    $root.omokoda.v1.AuditPassed.encode(message.auditPassed, writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                if (message.sabbathEntered != null && Object.hasOwnProperty.call(message, "sabbathEntered"))
                    $root.omokoda.v1.SabbathEntered.encode(message.sabbathEntered, writer.uint32(/* id 7, wireType 2 =*/58).fork()).ldelim();
                if (message.$unknowns != null && Object.hasOwnProperty.call(message, "$unknowns"))
                    for (var i = 0; i < message.$unknowns.length; ++i)
                        writer.raw(message.$unknowns[i]);
                return writer;
            };

            /**
             * Encodes the specified SovereignEvent message, length delimited. Does not implicitly {@link omokoda.v1.SovereignEvent.verify|verify} messages.
             * @function encodeDelimited
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {omokoda.v1.SovereignEvent.$Properties} message SovereignEvent message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SovereignEvent.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a SovereignEvent message from the specified reader or buffer.
             * @function decode
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape} SovereignEvent
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SovereignEvent.decode = function decode(reader, length, _end, _depth, _target) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $Reader.recursionLimit)
                    throw Error("max depth exceeded");
                var end = length === undefined ? reader.len : reader.pos + length, message = _target || new $root.omokoda.v1.SovereignEvent();
                while (reader.pos < end) {
                    var start = reader.pos;
                    var tag = reader.tag();
                    if (tag === _end) {
                        _end = undefined;
                        break;
                    }
                    var wireType = tag & 7;
                    switch (tag >>>= 3) {
                    case 1: {
                            if (wireType !== 2)
                                break;
                            message.agentBorn = $root.omokoda.v1.AgentBorn.decode(reader, reader.uint32(), undefined, _depth + 1, message.agentBorn);
                            message.event = "agentBorn";
                            continue;
                        }
                    case 2: {
                            if (wireType !== 2)
                                break;
                            message.thoughtSealed = $root.omokoda.v1.ThoughtSealed.decode(reader, reader.uint32(), undefined, _depth + 1, message.thoughtSealed);
                            message.event = "thoughtSealed";
                            continue;
                        }
                    case 3: {
                            if (wireType !== 2)
                                break;
                            message.actExecuted = $root.omokoda.v1.ActExecuted.decode(reader, reader.uint32(), undefined, _depth + 1, message.actExecuted);
                            message.event = "actExecuted";
                            continue;
                        }
                    case 4: {
                            if (wireType !== 2)
                                break;
                            message.tocMinted = $root.omokoda.v1.TocMinted.decode(reader, reader.uint32(), undefined, _depth + 1, message.tocMinted);
                            message.event = "tocMinted";
                            continue;
                        }
                    case 5: {
                            if (wireType !== 2)
                                break;
                            message.tierAdvanced = $root.omokoda.v1.TierAdvanced.decode(reader, reader.uint32(), undefined, _depth + 1, message.tierAdvanced);
                            message.event = "tierAdvanced";
                            continue;
                        }
                    case 6: {
                            if (wireType !== 2)
                                break;
                            message.auditPassed = $root.omokoda.v1.AuditPassed.decode(reader, reader.uint32(), undefined, _depth + 1, message.auditPassed);
                            message.event = "auditPassed";
                            continue;
                        }
                    case 7: {
                            if (wireType !== 2)
                                break;
                            message.sabbathEntered = $root.omokoda.v1.SabbathEntered.decode(reader, reader.uint32(), undefined, _depth + 1, message.sabbathEntered);
                            message.event = "sabbathEntered";
                            continue;
                        }
                    }
                    reader.skipType(wireType, _depth, tag);
                    $util.makeProp(message, "$unknowns", false);
                    (message.$unknowns || (message.$unknowns = [])).push(reader.raw(start, reader.pos));
                }
                if (_end !== undefined)
                    throw Error("missing end group");
                return message;
            };

            /**
             * Decodes a SovereignEvent message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape} SovereignEvent
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SovereignEvent.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a SovereignEvent message.
             * @function verify
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            SovereignEvent.verify = function verify(message, _depth) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    return "max depth exceeded";
                var properties = {};
                if (message.agentBorn != null && message.hasOwnProperty("agentBorn")) {
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.AgentBorn.verify(message.agentBorn, _depth + 1);
                        if (error)
                            return "agentBorn." + error;
                    }
                }
                if (message.thoughtSealed != null && message.hasOwnProperty("thoughtSealed")) {
                    if (properties.event === 1)
                        return "event: multiple values";
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.ThoughtSealed.verify(message.thoughtSealed, _depth + 1);
                        if (error)
                            return "thoughtSealed." + error;
                    }
                }
                if (message.actExecuted != null && message.hasOwnProperty("actExecuted")) {
                    if (properties.event === 1)
                        return "event: multiple values";
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.ActExecuted.verify(message.actExecuted, _depth + 1);
                        if (error)
                            return "actExecuted." + error;
                    }
                }
                if (message.tocMinted != null && message.hasOwnProperty("tocMinted")) {
                    if (properties.event === 1)
                        return "event: multiple values";
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.TocMinted.verify(message.tocMinted, _depth + 1);
                        if (error)
                            return "tocMinted." + error;
                    }
                }
                if (message.tierAdvanced != null && message.hasOwnProperty("tierAdvanced")) {
                    if (properties.event === 1)
                        return "event: multiple values";
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.TierAdvanced.verify(message.tierAdvanced, _depth + 1);
                        if (error)
                            return "tierAdvanced." + error;
                    }
                }
                if (message.auditPassed != null && message.hasOwnProperty("auditPassed")) {
                    if (properties.event === 1)
                        return "event: multiple values";
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.AuditPassed.verify(message.auditPassed, _depth + 1);
                        if (error)
                            return "auditPassed." + error;
                    }
                }
                if (message.sabbathEntered != null && message.hasOwnProperty("sabbathEntered")) {
                    if (properties.event === 1)
                        return "event: multiple values";
                    properties.event = 1;
                    {
                        var error = $root.omokoda.v1.SabbathEntered.verify(message.sabbathEntered, _depth + 1);
                        if (error)
                            return "sabbathEntered." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a SovereignEvent message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {omokoda.v1.SovereignEvent} SovereignEvent
             */
            SovereignEvent.fromObject = function fromObject(object, _depth) {
                if (object instanceof $root.omokoda.v1.SovereignEvent)
                    return object;
                if (_depth === undefined)
                    _depth = 0;
                if (_depth > $util.recursionLimit)
                    throw Error("max depth exceeded");
                var message = new $root.omokoda.v1.SovereignEvent();
                if (object.agentBorn != null) {
                    if (typeof object.agentBorn !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.agentBorn: object expected");
                    message.agentBorn = $root.omokoda.v1.AgentBorn.fromObject(object.agentBorn, _depth + 1);
                }
                if (object.thoughtSealed != null) {
                    if (typeof object.thoughtSealed !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.thoughtSealed: object expected");
                    message.thoughtSealed = $root.omokoda.v1.ThoughtSealed.fromObject(object.thoughtSealed, _depth + 1);
                }
                if (object.actExecuted != null) {
                    if (typeof object.actExecuted !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.actExecuted: object expected");
                    message.actExecuted = $root.omokoda.v1.ActExecuted.fromObject(object.actExecuted, _depth + 1);
                }
                if (object.tocMinted != null) {
                    if (typeof object.tocMinted !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.tocMinted: object expected");
                    message.tocMinted = $root.omokoda.v1.TocMinted.fromObject(object.tocMinted, _depth + 1);
                }
                if (object.tierAdvanced != null) {
                    if (typeof object.tierAdvanced !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.tierAdvanced: object expected");
                    message.tierAdvanced = $root.omokoda.v1.TierAdvanced.fromObject(object.tierAdvanced, _depth + 1);
                }
                if (object.auditPassed != null) {
                    if (typeof object.auditPassed !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.auditPassed: object expected");
                    message.auditPassed = $root.omokoda.v1.AuditPassed.fromObject(object.auditPassed, _depth + 1);
                }
                if (object.sabbathEntered != null) {
                    if (typeof object.sabbathEntered !== "object")
                        throw TypeError(".omokoda.v1.SovereignEvent.sabbathEntered: object expected");
                    message.sabbathEntered = $root.omokoda.v1.SabbathEntered.fromObject(object.sabbathEntered, _depth + 1);
                }
                return message;
            };

            /**
             * Creates a plain object from a SovereignEvent message. Also converts values to other types if specified.
             * @function toObject
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {omokoda.v1.SovereignEvent} message SovereignEvent
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            SovereignEvent.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (message.agentBorn != null && message.hasOwnProperty("agentBorn")) {
                    object.agentBorn = $root.omokoda.v1.AgentBorn.toObject(message.agentBorn, options);
                    if (options.oneofs)
                        object.event = "agentBorn";
                }
                if (message.thoughtSealed != null && message.hasOwnProperty("thoughtSealed")) {
                    object.thoughtSealed = $root.omokoda.v1.ThoughtSealed.toObject(message.thoughtSealed, options);
                    if (options.oneofs)
                        object.event = "thoughtSealed";
                }
                if (message.actExecuted != null && message.hasOwnProperty("actExecuted")) {
                    object.actExecuted = $root.omokoda.v1.ActExecuted.toObject(message.actExecuted, options);
                    if (options.oneofs)
                        object.event = "actExecuted";
                }
                if (message.tocMinted != null && message.hasOwnProperty("tocMinted")) {
                    object.tocMinted = $root.omokoda.v1.TocMinted.toObject(message.tocMinted, options);
                    if (options.oneofs)
                        object.event = "tocMinted";
                }
                if (message.tierAdvanced != null && message.hasOwnProperty("tierAdvanced")) {
                    object.tierAdvanced = $root.omokoda.v1.TierAdvanced.toObject(message.tierAdvanced, options);
                    if (options.oneofs)
                        object.event = "tierAdvanced";
                }
                if (message.auditPassed != null && message.hasOwnProperty("auditPassed")) {
                    object.auditPassed = $root.omokoda.v1.AuditPassed.toObject(message.auditPassed, options);
                    if (options.oneofs)
                        object.event = "auditPassed";
                }
                if (message.sabbathEntered != null && message.hasOwnProperty("sabbathEntered")) {
                    object.sabbathEntered = $root.omokoda.v1.SabbathEntered.toObject(message.sabbathEntered, options);
                    if (options.oneofs)
                        object.event = "sabbathEntered";
                }
                return object;
            };

            /**
             * Converts this SovereignEvent to JSON.
             * @function toJSON
             * @memberof omokoda.v1.SovereignEvent
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            SovereignEvent.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the type url for SovereignEvent
             * @function getTypeUrl
             * @memberof omokoda.v1.SovereignEvent
             * @static
             * @param {string} [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns {string} The type url
             */
            SovereignEvent.getTypeUrl = function getTypeUrl(prefix) {
                if (prefix === undefined)
                    prefix = "type.googleapis.com";
                return prefix + "/omokoda.v1.SovereignEvent";
            };

            return SovereignEvent;
        })();

        return v1;
    })();

    return omokoda;
})();

module.exports = $root;
