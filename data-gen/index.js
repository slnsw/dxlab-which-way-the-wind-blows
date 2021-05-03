require("dotenv").config();
const fs = require("fs").promises;
const axios = require("axios");
const eachDayOfInterval = require("date-fns/eachDayOfInterval");
const addDays = require("date-fns/addDays");
const _ = require("lodash");

const DAYS_OF_DATA = 7;
const CURVE_DEGREE = 3;

const ACCESS_TOKEN = process.env.ACCESS_TOKEN;

if (!ACCESS_TOKEN) {
    console.error(
        "An access token is required to acess the social media archive API. Please provide it via the ACCESS_TOKEN environment variable."
    );
    process.exit(1);
}

const getActivitesUrl = date =>
    `https://socialmediaarchive.sl.nsw.gov.au/api/activities?access_token=${ACCESS_TOKEN}&set=slnsw&toDate=${date
        .toISOString()
        .slice(0, 10)}`;

const DIVIDER = 50;

const scaleFunc = maxValue => v => {
    let f = 1 - v / maxValue;
    let curved = f ** CURVE_DEGREE;
    let inv = 1 - curved;
    return Math.round((inv * maxValue) / DIVIDER);
};

async function main() {
    const [dateArg] = process.argv.slice(2);

    let endDate, startDate;

    if (dateArg) {
        startDate = new Date(dateArg);
        endDate = addDays(startDate, DAYS_OF_DATA);
    } else {
        console.log("No date provided. Doing week until today.");
        endDate = new Date();
        startDate = addDays(endDate, -DAYS_OF_DATA);
    }

    const days = eachDayOfInterval({
        start: startDate,
        end: endDate,
    });

    const promises = days.map(async day => {
        const res = await axios(getActivitesUrl(day));
        return res.data.items;
    });

    const dataByDay = await Promise.all(promises);

    const allKeys = _.uniq(_.flatten(dataByDay.map(d => d.map(i => i.key))));
    let maxValue = 0;
    const groups = allKeys.map((key, index) => {
        const day_values = _.range(0, DAYS_OF_DATA + 1).map(i => {
            const dayValue = dataByDay[i].find(d => d.key === key);
            return dayValue ? Math.floor(dayValue.count) : 0;
        });
        maxValue = Math.max(maxValue, ...day_values);
        return {
            key,
            index,
            day_values,
        };
    });
    const scale = scaleFunc(maxValue);

    const scaledGroups = groups.map(g => ({
        ...g,
        display_values: g.day_values,
        day_values: g.day_values.map(scale),
    }));

    const output = {
        start_date: startDate.toLocaleDateString().replace(/\//g, "-"),
        end_date: endDate.toLocaleDateString().replace(/\//g, "-"),
        groups: scaledGroups,
    };

    await fs.writeFile("../data.json", JSON.stringify(output), { encoding: "utf-8" });
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
