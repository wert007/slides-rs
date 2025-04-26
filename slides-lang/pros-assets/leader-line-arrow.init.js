const namespace = 'deadlocks_problem-base';
const opts = {
    parent: document.getElementById(`${namespace}`),
    color: "black",
    path: "grid",
};
new LeaderLine(
    document.getElementById(`${namespace}-amanda`),
    document.getElementById(`${namespace}-lift`),
    opts
);
new LeaderLine(
    document.getElementById(`${namespace}-amanda`),
    document.getElementById(`${namespace}-light`),
    opts
);
new LeaderLine(
    document.getElementById(`${namespace}-bobbl`),
    document.getElementById(`${namespace}-lift`),
    opts
);
new LeaderLine(
    document.getElementById(`${namespace}-bobbl`),
    document.getElementById(`${namespace}-light`),
    opts
);
