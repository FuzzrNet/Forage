# Forage

> *Forage is for Storage*

![Tree being struck by lightning (royalty-free stock photo from pixabay)](forage.jpg)

- Remote storage: Open storage channels to a remote storage provider over Tor
- Lightweight: Platform-optimized using Blake3-based hash, verified streaming, and encryption algorithms
- Local files can be deleted: Periodic verification of remotely stored data
- Remote files can be retrieved: Files can be proven to exist remotely and sent back
- Low write-amplification: Bytes stored are only slightly larger than bytes read

**Caution!** This is experimental, potentially dangerous technology that hasn't yet been audited!

## Dependencies

- Embedded
    - [Blake3 cryptographic hash algorithm](https://github.com/BLAKE3-team/BLAKE3)
    - [Bao Verified Streaming for Provable Data Possession](https://github.com/oconnor663/bao)
    - [XChaCha8Blake3Siv encryption](https://github.com/PaulGrandperrin/XChaCha8Blake3Siv)
    - [Torut Tor controller](https://lib.rs/crates/torut)
    - [zstd_safe dictionary compression](https://github.com/gyscos/zstd-rs)
    - [Rusqlite embedded database](https://github.com/rusqlite/rusqlite)
- External
    - Local Tor SOCKS proxy

## Roadmap

### 0.0.1 - Experiment

- [x] Bao encoding
- [x] Bao verification
- [x] Bao extraction

### 0.0.2 - Persistence

- [ ] Append new files to central log file
- [ ] Extraction from appended log

### 0.0.3 - Tor

- [ ] Generate Onion v3 address
- [ ] Open & Receive TCP socket over Tor hidden service

### 0.1.0 - Proof of Concept

Goal: A storage client that can compress, encrypt, and store data on a remote storage provider using Tor. The storage client can check periodically that the data is still present and consistent on the remote storage provider against only a local 32-byte Blake3 hash without a full local reference copy, allowing the client to delete its local data, trusting that it can retrieve it later in-full. The storage client can then retrieve the data from the storage provider and decode it on-disk.

- [ ] Storage client can open a storage channel to storage provider over Tor
    - [ ] Storage provider generates Onion v3 address to provide to storage client out-of-band
    - [ ] Storage client generates Onion v3 address of their own
    - [ ] TCP socket is established from storage client to storage provider over Tor hidden service
- [ ] Storage client can store data on storage provider
    - [ ] Storage client can supply their node with specified path to data to store remotely
    - [ ] Data is encoded using Bao, hashed with Blake3, and transmitted over TCP socket over Tor circuit
    - [ ] Blake3 hash is persisted locally
    - [ ] Optional: Delete the local data
- [ ] Storage client can periodically verify the data they sent is still present and consistent over time
    - [ ] Storage client asks for a 4KB slice of data at a random offset of their choosing from the storage storage provider
    - [ ] Storage client checks 4KB slice against the same offset against local Bao Blake3 hash
- [ ] Storage client can retrieve data from storage provider over storage channel
    - [ ] Data is written to disk at specified path

### 0.1.1

Focus: Security

- [ ] Authentication between storage client and storage provider using Onion v3 addresses
- [ ] Files are encrypted using XChaCha8Blake3Siv
    - **Caution!** Experimental encryption!

### 0.2.0 - MVP

Focus: UX

- [ ] Can add files to an existing storage channel without a local copy of data by decoding local Blake3 digest and providing additional bytes
    - [ ] Appended bytes can be verified in-full by storage provider
- [ ] Multiple files can be stored
- [ ] Paths are indexed in sqlite (path, file name, file size, creation & modification dates, file hash)
- [ ] Files are compressed using zstd dictionary compression
- [ ] Individual files can be retrieved from storage client's log on storage provider

### 0.2.1

- [ ] Log files can be compacted and offset truncation accounted for

### Future

After this basic functionality exists, more exciting features are planned!
