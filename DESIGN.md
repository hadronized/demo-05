# Demoscene production design
> The goal of this document is to provide hindsight about how the demo / the system powering the demo got written in
> the first place. It contains various information about technical choices but also architecture and overall design
> choices.

<!-- vim-markdown-toc GFM -->

* [Overall architecture](#overall-architecture)
  * [Graphics system](#graphics-system)
  * [Audio system](#audio-system)
  * [Synchronizer system](#synchronizer-system)
  * [Entity system](#entity-system)
    * [Mapping entities](#mapping-entities)
  * [Runtime system](#runtime-system)
  * [Protocol library](#protocol-library)
  * [System library](#system-library)

<!-- vim-markdown-toc -->

# Overall architecture
> This section describes the overall design of the demo in functional terms.

The production is organized in several _systems_. A system is an isolated piece of software that can receive signals from
other systems via a mechanism of _messages_. Messages allow systems to communicate between each other via a typed
interface. Not all systems are connected to each other, and the interaction is done entirely by the _runtime system_.

## Graphics system
> This section describes the graphics part of the demo, which is responsible for showing stuff to the user on, typically
> their screens.

## Audio system
> This seciton describes the audio part of the engine, which is responsible for playing the soundtrack and providing the
> rest of the engine audio features such as cursors, toggling back and forth the audio, and play some sounds.

## Synchronizer system
> This section describes the part of the engine that is responsible for synchronizing the graphics with the audio. It is
> responsible for advancing the “step” of the demo from its beginning to its end.

## Entity system
> This section describes the assets mechanism, which is responsible for loading assets from the storage and reacting to
> asset changes. It is especially missioned to control the streaming of assets.

The way the _entity system_ works is simple: each entity is declared via a type. The entity system will be able to
load, stream and watch an entity if it appears in its mapping setting.

### Mapping entities

A type `EntityA` is managed if it is added in the `Entity` _enum_ and that exists at least one path from a
_representation_ to `Entity::EntityA`.

Representations are a way to be able to load, stream and watch `EntityA` for different kind of sources and formats. The
way this is done is by adding some code in the format dispatcher:

1. Extensions are matched: for instance, for `.obj` files, we can treat them as `EntityA` by invoking the right parser.
2. Some resources share the same format, such as _JSON_, _TOML_, etc. In those cases, `EntityA` could also be
  represented via JSON. In such a situation, a code path must be added in the right format dispatcher.

On a general note, a type of resource can be loaded from different _sources_. The way the entity system works is
_event-based_, which means the user doesn’t ask to load a resource; they just ask for it without caring about the
fact it is formatted in JSON or TOML.

For this reason, once an entity is loaded, it is assigned a name other systems expect to find — it’s not its path on
the filesystem. That name is a unique identifier for that resource and must be extracted from the resource in whatever
way fits the best. The mapping is known only by the entity system.

## Runtime system
> This section describes the code that puts everything together to yield the final executable.

## Protocol library
> This section is about the protocol library, which is not a system but more a shared library providing
> protocols that can be used by other systems to agree on what to do with a value.

## System library
> This library provides a way to create systems, used by other parts to actually declare their systems, etc.
