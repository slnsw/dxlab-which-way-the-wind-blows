# Which Way The Wind Blows (DX Lab Digital Drop-in Program)

Which Way The Wind Blows presents a novel way to visualise patterns in social media posts. Leveraging on people's familiarity with nightly weather reports, topics are visualised as evolving pressure systems, volume of posts as isobars, and notable keywords as troughs. Revealing what people talk and care about on social media can become a crucial component in understanding what life is like in the state of New South Wales.

The State Library of NSW's Social Media Archive collects material to provide a documentary record of life for the state's citizens. The archive has established itself as a rich and versatile resource for researchers and domain experts.

Which Way The Wind Blows aims to improve accessibility, interpretation and appreciation of The Social Media Archive to the general public. We hope that by 'casualising' the Social Media Archive data, we can promote awareness, encourage utilisation, and build an ecosystem for innovation around the archive - making the archive an integral part of the State Library collection.

Project Credits:

-   Concept/design: Chuan Jia (Jack) Zhao
-   Creative Technologist: Harry Morris

A Small Multiples Project
[small.mu](https://small.mu)

## Setup

```
cd data-gen
yarn
```

## Fetching data

`node index.js <start date or blank>`

-   if you provide an argument for date - it is the start date and then end date is a week later
-   if you dont - end date is today and start is a week earlier
-   date anything javascript accepts, its pretty smart but YYYY-MM-DD is safe 2020-01-06
-   it'll output frames to a directory ./out/$start_date/$frame_no.png - ill get you the ffmpeg script to make this a video

## Rendering

```
cd nanno-fluid-sim
cargo run --release
```

## Converting images to video

In the render output directory, run
`ffmpeg -framerate 30 -i %06d.png -c:v libx264 -pix_fmt yuv420p ../my-cool-video.mp4`

To remove jitter, double the speed of the video
`ffmpeg -i my-cool-video.mp4 -filter:v "setpts=0.5*PTS" my-cool-video-2x.mp4`
