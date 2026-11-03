//! Help and usage texts, generated from the
//! Python's own argparse rendering at 80 columns
//! (the oracle's -h output), pinned as constants.

pub const TOP_HELP: &str = "usage: onrecord [-h] {add,get,subject,due,check} ...\n\npositional arguments:\n  {add,get,subject,due,check}\n    add                 append a claim\n    get                 retrieve a claim by id\n    subject             retrieve a subject\n    due                 list the live claims due by a date\n    check               name every defect\n\noptions:\n  -h, --help            show this help message and exit\n";

pub const TOP_USAGE: &str = "usage: onrecord [-h] {add,get,subject,due,check} ...\n";

pub const ADD_HELP: &str = "usage: onrecord add [-h] --text TEXT --asserter ASSERTER --at AT\n                    [--evidence EVIDENCE] [--supersedes SUPERSEDES]\n                    [--subject SUBJECT] [--due DUE]\n                    record\n\npositional arguments:\n  record\n\noptions:\n  -h, --help            show this help message and exit\n  --text TEXT\n  --asserter ASSERTER\n  --at AT\n  --evidence EVIDENCE\n  --supersedes SUPERSEDES\n  --subject SUBJECT\n  --due DUE\n";

pub const ADD_USAGE: &str = "usage: onrecord add [-h] --text TEXT --asserter ASSERTER --at AT\n                    [--evidence EVIDENCE] [--supersedes SUPERSEDES]\n                    [--subject SUBJECT] [--due DUE]\n                    record\n";

pub const GET_HELP: &str = "usage: onrecord get [-h] record id\n\npositional arguments:\n  record\n  id\n\noptions:\n  -h, --help  show this help message and exit\n";

pub const GET_USAGE: &str = "usage: onrecord get [-h] record id\n";

pub const SUBJECT_HELP: &str = "usage: onrecord subject [-h] record key\n\npositional arguments:\n  record\n  key\n\noptions:\n  -h, --help  show this help message and exit\n";

pub const SUBJECT_USAGE: &str = "usage: onrecord subject [-h] record key\n";

pub const DUE_HELP: &str = "usage: onrecord due [-h] record date\n\npositional arguments:\n  record\n  date\n\noptions:\n  -h, --help  show this help message and exit\n";

pub const DUE_USAGE: &str = "usage: onrecord due [-h] record date\n";

pub const CHECK_HELP: &str = "usage: onrecord check [-h] record\n\npositional arguments:\n  record\n\noptions:\n  -h, --help  show this help message and exit\n";

pub const CHECK_USAGE: &str = "usage: onrecord check [-h] record\n";
