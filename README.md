# Demoscene production design
> The goal of this document is to provide hindsight about how the demo / the system powering the demo got written in
> the first place. It contains various information about technical choices but also architecture and overall design
> choices.

<!-- vim-markdown-toc GFM -->

* [Overall architecture](#overall-architecture)
  * [Graphics layer](#graphics-layer)
  * [Audio layer](#audio-layer)
  * [Synchronizer layer](#synchronizer-layer)
  * [Assets layer](#assets-layer)

<!-- vim-markdown-toc -->

## Overall architecture
> This section describes the overall design of the demo in functional terms.

### Graphics layer
> This section describes the graphics part of the demo, which is responsible for showing stuff to the user on, typically
> their screens.

### Audio layer
> This seciton describes the audio part of the engine, which is responsible for playing the soundtrack and providing the
> rest of the engine audio features such as cursors, toggling back and forth the audio, and play some sounds.

### Synchronizer layer
> This section describes the part of the engine that is responsible for synchronizing the graphics with the audio. It is
> responsible for advancing the “step” of the demo from its beginning to its end.

### Assets layer
> This section describes the assets mechanism, which is responsible for loading assets from the storage and reacting to
> asset changes. It is especially missioned to control the streaming of assets.
