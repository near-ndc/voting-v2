import { parseNearAmount } from "near-api-js/lib/utils/format.js";
import { snapshot } from "../index.js";
import { BN } from 'bn.js';

const NEAR_THRESHOLD = new BN(parseNearAmount('1000'));
const NEAR_TO_YOCTO = new BN(parseNearAmount('1'));

const stakePower = (stakeStr) => {
    const stake = new BN(stakeStr);

    let stake_power = stake.div(NEAR_TO_YOCTO).toNumber();

    if (stake.gt(NEAR_THRESHOLD)) {
        stake_power = 1000;
        stake_power += Math.floor(Math.sqrt(stake.sub(NEAR_THRESHOLD).div(NEAR_TO_YOCTO).toNumber()));
    }
    return stake_power;

}

const activityPower = (activeMonths) => {
    return activeMonths * 20;
}

const userToVotePower = (user) => {
    const activeMonthsPower = activityPower(user.active_months);
    const stake_power = stakePower(user.stake);

    return activeMonthsPower + stake_power;
}

export const GetSnapshot = (req, res) => {
    const { page = 0, limit = 100, sortBy = "name", sortOrder = "asc", prefix = "" } = req.query;

    // Convert page and limit to numbers
    const pageNumber = parseInt(page);
    const limitNumber = parseInt(limit);

    let filteredSnapshot = snapshot;
    if (prefix) {
        filteredSnapshot = snapshot.filter((item) => item.account_id.startsWith(prefix));
    }

    // Sort the snapshot data based on the sortBy and sortOrder parameters
    let sortedSnapshot = [...filteredSnapshot];
    switch (sortBy) {
        case "name":
            sortedSnapshot.sort((a, b) => a.account_id.localeCompare(b.account_id));
            break;
        case "stake":
            sortedSnapshot.sort((a, b) => {
                const stakeA = new BN(a.stake);
                const stakeB = new BN(b.stake);
                return stakeA.cmp(stakeB);
            });
            break;
        case "active_months":
            sortedSnapshot.sort((a, b) => a.active_months - b.active_months);
            break;
        case "vote_power":
            sortedSnapshot = sortedSnapshot.map(a => ({ power: userToVotePower(a), ...a })).sort((a, b) => a.power - b.power);
            break;
        case "stake_power":
            sortedSnapshot = sortedSnapshot.map(a => ({ power: stakePower(a.stake), ...a })).sort((a, b) => a.power - b.power);
            break;
        case "activity_power":
            sortedSnapshot = sortedSnapshot.map(a => ({ power: activityPower(a.active_months), ...a })).sort((a, b) => a.power - b.power);
            break;
        default:
            res.status(400).send(`Invalid sortBy parameter: ${sortBy}`);
            break;
    }

    if (sortOrder === "desc") {
        sortedSnapshot.reverse();
    }

    const startIndex = pageNumber * limitNumber;
    const endIndex = startIndex + limitNumber;
    const paginatedData = sortedSnapshot.slice(startIndex, endIndex);

    // Prepare the response
    const response = {
        data: paginatedData,
        currentPage: pageNumber + 1,
        totalPages: Math.ceil(sortedSnapshot.length / limitNumber),
        totalItems: sortedSnapshot.length,
    };

    res.json(response);
};
