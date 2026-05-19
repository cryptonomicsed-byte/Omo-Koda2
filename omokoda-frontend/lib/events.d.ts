import * as $protobuf from "protobufjs";
import Long = require("long");

/** Namespace omokoda. */
export namespace omokoda {

    /** Namespace v1. */
    namespace v1 {

        /**
         * Properties of an AgentBorn.
         * @deprecated Use omokoda.v1.AgentBorn.$Properties instead.
         */
        interface IAgentBorn extends omokoda.v1.AgentBorn.$Properties {
        }

        /** Represents an AgentBorn. */
        class AgentBorn {

            /**
             * Constructs a new AgentBorn.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.AgentBorn.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** AgentBorn dna. */
            dna: string;

            /** AgentBorn mnemonic. */
            mnemonic: string[];

            /** AgentBorn odu. */
            odu: number;

            /**
             * Creates a new AgentBorn instance using the specified properties.
             * @param [properties] Properties to set
             * @returns AgentBorn instance
             */
            static create(properties: omokoda.v1.AgentBorn.$Shape): omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape;
            static create(properties?: omokoda.v1.AgentBorn.$Properties): omokoda.v1.AgentBorn;

            /**
             * Encodes the specified AgentBorn message. Does not implicitly {@link omokoda.v1.AgentBorn.verify|verify} messages.
             * @param message AgentBorn message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.AgentBorn.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified AgentBorn message, length delimited. Does not implicitly {@link omokoda.v1.AgentBorn.verify|verify} messages.
             * @param message AgentBorn message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.AgentBorn.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an AgentBorn message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape} AgentBorn
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape;

            /**
             * Decodes an AgentBorn message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape} AgentBorn
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.AgentBorn & omokoda.v1.AgentBorn.$Shape;

            /**
             * Verifies an AgentBorn message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an AgentBorn message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns AgentBorn
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.AgentBorn;

            /**
             * Creates a plain object from an AgentBorn message. Also converts values to other types if specified.
             * @param message AgentBorn
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.AgentBorn, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this AgentBorn to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for AgentBorn
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace AgentBorn {

            /** Properties of an AgentBorn. */
            interface $Properties {

                /** AgentBorn dna */
                dna?: (string|null);

                /** AgentBorn mnemonic */
                mnemonic?: (string[]|null);

                /** AgentBorn odu */
                odu?: (number|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of an AgentBorn. */
            type $Shape = omokoda.v1.AgentBorn.$Properties;
        }

        /**
         * Properties of a ThoughtSealed.
         * @deprecated Use omokoda.v1.ThoughtSealed.$Properties instead.
         */
        interface IThoughtSealed extends omokoda.v1.ThoughtSealed.$Properties {
        }

        /** Represents a ThoughtSealed. */
        class ThoughtSealed {

            /**
             * Constructs a new ThoughtSealed.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.ThoughtSealed.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** ThoughtSealed intentHash. */
            intentHash: Uint8Array;

            /** ThoughtSealed hermeticScore. */
            hermeticScore: number;

            /**
             * Creates a new ThoughtSealed instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ThoughtSealed instance
             */
            static create(properties: omokoda.v1.ThoughtSealed.$Shape): omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape;
            static create(properties?: omokoda.v1.ThoughtSealed.$Properties): omokoda.v1.ThoughtSealed;

            /**
             * Encodes the specified ThoughtSealed message. Does not implicitly {@link omokoda.v1.ThoughtSealed.verify|verify} messages.
             * @param message ThoughtSealed message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.ThoughtSealed.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ThoughtSealed message, length delimited. Does not implicitly {@link omokoda.v1.ThoughtSealed.verify|verify} messages.
             * @param message ThoughtSealed message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.ThoughtSealed.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ThoughtSealed message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape} ThoughtSealed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape;

            /**
             * Decodes a ThoughtSealed message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape} ThoughtSealed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.ThoughtSealed & omokoda.v1.ThoughtSealed.$Shape;

            /**
             * Verifies a ThoughtSealed message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ThoughtSealed message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ThoughtSealed
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.ThoughtSealed;

            /**
             * Creates a plain object from a ThoughtSealed message. Also converts values to other types if specified.
             * @param message ThoughtSealed
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.ThoughtSealed, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ThoughtSealed to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for ThoughtSealed
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace ThoughtSealed {

            /** Properties of a ThoughtSealed. */
            interface $Properties {

                /** ThoughtSealed intentHash */
                intentHash?: (Uint8Array|null);

                /** ThoughtSealed hermeticScore */
                hermeticScore?: (number|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of a ThoughtSealed. */
            type $Shape = omokoda.v1.ThoughtSealed.$Properties;
        }

        /**
         * Properties of an ActExecuted.
         * @deprecated Use omokoda.v1.ActExecuted.$Properties instead.
         */
        interface IActExecuted extends omokoda.v1.ActExecuted.$Properties {
        }

        /** Represents an ActExecuted. */
        class ActExecuted {

            /**
             * Constructs a new ActExecuted.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.ActExecuted.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** ActExecuted tool. */
            tool: string;

            /** ActExecuted receiptMerkle. */
            receiptMerkle: Uint8Array;

            /** ActExecuted f1Score. */
            f1Score: number;

            /**
             * Creates a new ActExecuted instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ActExecuted instance
             */
            static create(properties: omokoda.v1.ActExecuted.$Shape): omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape;
            static create(properties?: omokoda.v1.ActExecuted.$Properties): omokoda.v1.ActExecuted;

            /**
             * Encodes the specified ActExecuted message. Does not implicitly {@link omokoda.v1.ActExecuted.verify|verify} messages.
             * @param message ActExecuted message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.ActExecuted.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ActExecuted message, length delimited. Does not implicitly {@link omokoda.v1.ActExecuted.verify|verify} messages.
             * @param message ActExecuted message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.ActExecuted.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an ActExecuted message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape} ActExecuted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape;

            /**
             * Decodes an ActExecuted message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape} ActExecuted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.ActExecuted & omokoda.v1.ActExecuted.$Shape;

            /**
             * Verifies an ActExecuted message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an ActExecuted message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ActExecuted
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.ActExecuted;

            /**
             * Creates a plain object from an ActExecuted message. Also converts values to other types if specified.
             * @param message ActExecuted
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.ActExecuted, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ActExecuted to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for ActExecuted
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace ActExecuted {

            /** Properties of an ActExecuted. */
            interface $Properties {

                /** ActExecuted tool */
                tool?: (string|null);

                /** ActExecuted receiptMerkle */
                receiptMerkle?: (Uint8Array|null);

                /** ActExecuted f1Score */
                f1Score?: (number|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of an ActExecuted. */
            type $Shape = omokoda.v1.ActExecuted.$Properties;
        }

        /**
         * Properties of a TocMinted.
         * @deprecated Use omokoda.v1.TocMinted.$Properties instead.
         */
        interface ITocMinted extends omokoda.v1.TocMinted.$Properties {
        }

        /** Represents a TocMinted. */
        class TocMinted {

            /**
             * Constructs a new TocMinted.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.TocMinted.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** TocMinted agent. */
            agent: string;

            /** TocMinted dopamineBurned. */
            dopamineBurned: (number|Long);

            /** TocMinted synapseEarned. */
            synapseEarned: (number|Long);

            /**
             * Creates a new TocMinted instance using the specified properties.
             * @param [properties] Properties to set
             * @returns TocMinted instance
             */
            static create(properties: omokoda.v1.TocMinted.$Shape): omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape;
            static create(properties?: omokoda.v1.TocMinted.$Properties): omokoda.v1.TocMinted;

            /**
             * Encodes the specified TocMinted message. Does not implicitly {@link omokoda.v1.TocMinted.verify|verify} messages.
             * @param message TocMinted message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.TocMinted.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified TocMinted message, length delimited. Does not implicitly {@link omokoda.v1.TocMinted.verify|verify} messages.
             * @param message TocMinted message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.TocMinted.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a TocMinted message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape} TocMinted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape;

            /**
             * Decodes a TocMinted message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape} TocMinted
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.TocMinted & omokoda.v1.TocMinted.$Shape;

            /**
             * Verifies a TocMinted message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a TocMinted message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns TocMinted
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.TocMinted;

            /**
             * Creates a plain object from a TocMinted message. Also converts values to other types if specified.
             * @param message TocMinted
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.TocMinted, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this TocMinted to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for TocMinted
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace TocMinted {

            /** Properties of a TocMinted. */
            interface $Properties {

                /** TocMinted agent */
                agent?: (string|null);

                /** TocMinted dopamineBurned */
                dopamineBurned?: (number|Long|null);

                /** TocMinted synapseEarned */
                synapseEarned?: (number|Long|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of a TocMinted. */
            type $Shape = omokoda.v1.TocMinted.$Properties;
        }

        /**
         * Properties of a TierAdvanced.
         * @deprecated Use omokoda.v1.TierAdvanced.$Properties instead.
         */
        interface ITierAdvanced extends omokoda.v1.TierAdvanced.$Properties {
        }

        /** Represents a TierAdvanced. */
        class TierAdvanced {

            /**
             * Constructs a new TierAdvanced.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.TierAdvanced.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** TierAdvanced agent. */
            agent: string;

            /** TierAdvanced oldTier. */
            oldTier: number;

            /** TierAdvanced newTier. */
            newTier: number;

            /**
             * Creates a new TierAdvanced instance using the specified properties.
             * @param [properties] Properties to set
             * @returns TierAdvanced instance
             */
            static create(properties: omokoda.v1.TierAdvanced.$Shape): omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape;
            static create(properties?: omokoda.v1.TierAdvanced.$Properties): omokoda.v1.TierAdvanced;

            /**
             * Encodes the specified TierAdvanced message. Does not implicitly {@link omokoda.v1.TierAdvanced.verify|verify} messages.
             * @param message TierAdvanced message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.TierAdvanced.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified TierAdvanced message, length delimited. Does not implicitly {@link omokoda.v1.TierAdvanced.verify|verify} messages.
             * @param message TierAdvanced message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.TierAdvanced.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a TierAdvanced message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape} TierAdvanced
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape;

            /**
             * Decodes a TierAdvanced message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape} TierAdvanced
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.TierAdvanced & omokoda.v1.TierAdvanced.$Shape;

            /**
             * Verifies a TierAdvanced message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a TierAdvanced message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns TierAdvanced
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.TierAdvanced;

            /**
             * Creates a plain object from a TierAdvanced message. Also converts values to other types if specified.
             * @param message TierAdvanced
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.TierAdvanced, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this TierAdvanced to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for TierAdvanced
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace TierAdvanced {

            /** Properties of a TierAdvanced. */
            interface $Properties {

                /** TierAdvanced agent */
                agent?: (string|null);

                /** TierAdvanced oldTier */
                oldTier?: (number|null);

                /** TierAdvanced newTier */
                newTier?: (number|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of a TierAdvanced. */
            type $Shape = omokoda.v1.TierAdvanced.$Properties;
        }

        /**
         * Properties of an AuditPassed.
         * @deprecated Use omokoda.v1.AuditPassed.$Properties instead.
         */
        interface IAuditPassed extends omokoda.v1.AuditPassed.$Properties {
        }

        /** Represents an AuditPassed. */
        class AuditPassed {

            /**
             * Constructs a new AuditPassed.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.AuditPassed.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** AuditPassed receiptId. */
            receiptId: string;

            /** AuditPassed zangbetoSig. */
            zangbetoSig: Uint8Array;

            /**
             * Creates a new AuditPassed instance using the specified properties.
             * @param [properties] Properties to set
             * @returns AuditPassed instance
             */
            static create(properties: omokoda.v1.AuditPassed.$Shape): omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape;
            static create(properties?: omokoda.v1.AuditPassed.$Properties): omokoda.v1.AuditPassed;

            /**
             * Encodes the specified AuditPassed message. Does not implicitly {@link omokoda.v1.AuditPassed.verify|verify} messages.
             * @param message AuditPassed message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.AuditPassed.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified AuditPassed message, length delimited. Does not implicitly {@link omokoda.v1.AuditPassed.verify|verify} messages.
             * @param message AuditPassed message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.AuditPassed.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an AuditPassed message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape} AuditPassed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape;

            /**
             * Decodes an AuditPassed message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape} AuditPassed
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.AuditPassed & omokoda.v1.AuditPassed.$Shape;

            /**
             * Verifies an AuditPassed message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an AuditPassed message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns AuditPassed
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.AuditPassed;

            /**
             * Creates a plain object from an AuditPassed message. Also converts values to other types if specified.
             * @param message AuditPassed
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.AuditPassed, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this AuditPassed to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for AuditPassed
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace AuditPassed {

            /** Properties of an AuditPassed. */
            interface $Properties {

                /** AuditPassed receiptId */
                receiptId?: (string|null);

                /** AuditPassed zangbetoSig */
                zangbetoSig?: (Uint8Array|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of an AuditPassed. */
            type $Shape = omokoda.v1.AuditPassed.$Properties;
        }

        /**
         * Properties of a SabbathEntered.
         * @deprecated Use omokoda.v1.SabbathEntered.$Properties instead.
         */
        interface ISabbathEntered extends omokoda.v1.SabbathEntered.$Properties {
        }

        /** Represents a SabbathEntered. */
        class SabbathEntered {

            /**
             * Constructs a new SabbathEntered.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.SabbathEntered.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** SabbathEntered agentsPaused. */
            agentsPaused: string[];

            /** SabbathEntered queuedOps. */
            queuedOps: number;

            /**
             * Creates a new SabbathEntered instance using the specified properties.
             * @param [properties] Properties to set
             * @returns SabbathEntered instance
             */
            static create(properties: omokoda.v1.SabbathEntered.$Shape): omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape;
            static create(properties?: omokoda.v1.SabbathEntered.$Properties): omokoda.v1.SabbathEntered;

            /**
             * Encodes the specified SabbathEntered message. Does not implicitly {@link omokoda.v1.SabbathEntered.verify|verify} messages.
             * @param message SabbathEntered message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.SabbathEntered.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified SabbathEntered message, length delimited. Does not implicitly {@link omokoda.v1.SabbathEntered.verify|verify} messages.
             * @param message SabbathEntered message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.SabbathEntered.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a SabbathEntered message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape} SabbathEntered
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape;

            /**
             * Decodes a SabbathEntered message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape} SabbathEntered
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.SabbathEntered & omokoda.v1.SabbathEntered.$Shape;

            /**
             * Verifies a SabbathEntered message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a SabbathEntered message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns SabbathEntered
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.SabbathEntered;

            /**
             * Creates a plain object from a SabbathEntered message. Also converts values to other types if specified.
             * @param message SabbathEntered
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.SabbathEntered, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this SabbathEntered to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for SabbathEntered
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace SabbathEntered {

            /** Properties of a SabbathEntered. */
            interface $Properties {

                /** SabbathEntered agentsPaused */
                agentsPaused?: (string[]|null);

                /** SabbathEntered queuedOps */
                queuedOps?: (number|null);

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Shape of a SabbathEntered. */
            type $Shape = omokoda.v1.SabbathEntered.$Properties;
        }

        /**
         * Properties of a SovereignEvent.
         * @deprecated Use omokoda.v1.SovereignEvent.$Properties instead.
         */
        interface ISovereignEvent extends omokoda.v1.SovereignEvent.$Properties {
        }

        /** Represents a SovereignEvent. */
        class SovereignEvent {

            /**
             * Constructs a new SovereignEvent.
             * @param [properties] Properties to set
             */
            constructor(properties?: omokoda.v1.SovereignEvent.$Properties);

            /** Unknown fields preserved while decoding */
            $unknowns?: Uint8Array[];

            /** SovereignEvent agentBorn. */
            agentBorn?: (omokoda.v1.AgentBorn.$Properties|null);

            /** SovereignEvent thoughtSealed. */
            thoughtSealed?: (omokoda.v1.ThoughtSealed.$Properties|null);

            /** SovereignEvent actExecuted. */
            actExecuted?: (omokoda.v1.ActExecuted.$Properties|null);

            /** SovereignEvent tocMinted. */
            tocMinted?: (omokoda.v1.TocMinted.$Properties|null);

            /** SovereignEvent tierAdvanced. */
            tierAdvanced?: (omokoda.v1.TierAdvanced.$Properties|null);

            /** SovereignEvent auditPassed. */
            auditPassed?: (omokoda.v1.AuditPassed.$Properties|null);

            /** SovereignEvent sabbathEntered. */
            sabbathEntered?: (omokoda.v1.SabbathEntered.$Properties|null);

            /** SovereignEvent event. */
            event?: ("agentBorn"|"thoughtSealed"|"actExecuted"|"tocMinted"|"tierAdvanced"|"auditPassed"|"sabbathEntered");

            /**
             * Creates a new SovereignEvent instance using the specified properties.
             * @param [properties] Properties to set
             * @returns SovereignEvent instance
             */
            static create(properties: omokoda.v1.SovereignEvent.$Shape): omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape;
            static create(properties?: omokoda.v1.SovereignEvent.$Properties): omokoda.v1.SovereignEvent;

            /**
             * Encodes the specified SovereignEvent message. Does not implicitly {@link omokoda.v1.SovereignEvent.verify|verify} messages.
             * @param message SovereignEvent message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encode(message: omokoda.v1.SovereignEvent.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified SovereignEvent message, length delimited. Does not implicitly {@link omokoda.v1.SovereignEvent.verify|verify} messages.
             * @param message SovereignEvent message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            static encodeDelimited(message: omokoda.v1.SovereignEvent.$Properties, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a SovereignEvent message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns {omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape} SovereignEvent
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape;

            /**
             * Decodes a SovereignEvent message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns {omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape} SovereignEvent
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): omokoda.v1.SovereignEvent & omokoda.v1.SovereignEvent.$Shape;

            /**
             * Verifies a SovereignEvent message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a SovereignEvent message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns SovereignEvent
             */
            static fromObject(object: { [k: string]: any }): omokoda.v1.SovereignEvent;

            /**
             * Creates a plain object from a SovereignEvent message. Also converts values to other types if specified.
             * @param message SovereignEvent
             * @param [options] Conversion options
             * @returns Plain object
             */
            static toObject(message: omokoda.v1.SovereignEvent, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this SovereignEvent to JSON.
             * @returns JSON object
             */
            toJSON(): { [k: string]: any };

            /**
             * Gets the type url for SovereignEvent
             * @param [prefix] Custom type url prefix, defaults to `"type.googleapis.com"`
             * @returns The type url
             */
            static getTypeUrl(prefix?: string): string;
        }

        namespace SovereignEvent {

            /** Properties of a SovereignEvent. */
            interface $Properties {

                /** SovereignEvent agentBorn */
                agentBorn?: (omokoda.v1.AgentBorn.$Properties|null);

                /** SovereignEvent thoughtSealed */
                thoughtSealed?: (omokoda.v1.ThoughtSealed.$Properties|null);

                /** SovereignEvent actExecuted */
                actExecuted?: (omokoda.v1.ActExecuted.$Properties|null);

                /** SovereignEvent tocMinted */
                tocMinted?: (omokoda.v1.TocMinted.$Properties|null);

                /** SovereignEvent tierAdvanced */
                tierAdvanced?: (omokoda.v1.TierAdvanced.$Properties|null);

                /** SovereignEvent auditPassed */
                auditPassed?: (omokoda.v1.AuditPassed.$Properties|null);

                /** SovereignEvent sabbathEntered */
                sabbathEntered?: (omokoda.v1.SabbathEntered.$Properties|null);

                /** SovereignEvent event */
                event?: ("agentBorn"|"thoughtSealed"|"actExecuted"|"tocMinted"|"tierAdvanced"|"auditPassed"|"sabbathEntered");

                /** Unknown fields preserved while decoding */
                $unknowns?: Uint8Array[];
            }

            /** Narrowed shape of a SovereignEvent. */
            type $Shape = {
  agentBorn?: omokoda.v1.AgentBorn.$Shape|null;
  thoughtSealed?: omokoda.v1.ThoughtSealed.$Shape|null;
  actExecuted?: omokoda.v1.ActExecuted.$Shape|null;
  tocMinted?: omokoda.v1.TocMinted.$Shape|null;
  tierAdvanced?: omokoda.v1.TierAdvanced.$Shape|null;
  auditPassed?: omokoda.v1.AuditPassed.$Shape|null;
  sabbathEntered?: omokoda.v1.SabbathEntered.$Shape|null;
  $unknowns?: Uint8Array[];
} & (
  ({ event?: undefined; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "agentBorn"; agentBorn: omokoda.v1.AgentBorn.$Shape; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "thoughtSealed"; agentBorn?: null; thoughtSealed: omokoda.v1.ThoughtSealed.$Shape; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "actExecuted"; agentBorn?: null; thoughtSealed?: null; actExecuted: omokoda.v1.ActExecuted.$Shape; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "tocMinted"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted: omokoda.v1.TocMinted.$Shape; tierAdvanced?: null; auditPassed?: null; sabbathEntered?: null }|{ event?: "tierAdvanced"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced: omokoda.v1.TierAdvanced.$Shape; auditPassed?: null; sabbathEntered?: null }|{ event?: "auditPassed"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed: omokoda.v1.AuditPassed.$Shape; sabbathEntered?: null }|{ event?: "sabbathEntered"; agentBorn?: null; thoughtSealed?: null; actExecuted?: null; tocMinted?: null; tierAdvanced?: null; auditPassed?: null; sabbathEntered: omokoda.v1.SabbathEntered.$Shape })
);
        }
    }
}
