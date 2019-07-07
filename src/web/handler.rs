use super::AppState;
use actix_web as aweb;
use failure::Error;
use genpdf::dir_info;
use serde_json;
use std::sync::Arc;

pub fn index_state(req: &aweb::HttpRequest<Arc<AppState>>) -> String {
    // let count = req.state().counter.get() + 1; // <- get count
    // req.state().counter.set(count); // <- store new count in state

    // format!("Request number: {}", count) // <- response with count
    format!("Example removed")
}

pub fn index(_req: &aweb::HttpRequest) -> &'static str {
    "Hello world!"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dirs {
    dirs: Vec<dir_info::DirInfo>,
}

pub fn get_dirs(req: &aweb::HttpRequest<Arc<AppState>>) -> String {
    let dirs = Dirs {
        dirs: req.state().dirs.clone(),
    };
    serde_json::to_string(&dirs.dirs).expect(&fh!())
}

// TODO: this is insecure. The received information should deal with
// indexes only
pub fn gen_projs(
    (dirs, req): (aweb::Json<Dirs>, aweb::HttpRequest<Arc<AppState>>),
) -> aweb::Result<String> {
    ph!("gen proj!! {:?}", &dirs);
    for dir in &dirs.dirs {
        genpdf::temp::gen_proj(dir, &req.state().consts)?;
    }
    Ok("nice".into())
}

pub fn clear_projs(
    (dirs, req): (aweb::Json<Dirs>, aweb::HttpRequest<Arc<AppState>>),
) -> aweb::Result<String> {
    ph!("clear proj!! {:?}", &dirs);
    for dir in &dirs.dirs {
        genpdf::temp::clear_tmp(dir)?;
    }
    Ok("nice".into())
}

// TODO: properly move this into a proper Tera+bootstrap template
// and also ajax request into the get_dirs
pub fn temporary_index(req: &aweb::HttpRequest<Arc<AppState>>) -> aweb::HttpResponse {
    let dirs = get_dirs(req);

    let data = format!(r##"
        <!DOCTYPE html>
        <html>
        <head>

        <!-- thx Pedro Gonçalves! -->
        <meta charset="UTF-8">

        <link href="https://unpkg.com/tabulator-tables@4.2.3/dist/css/tabulator_midnight.min.css" rel="stylesheet">
        <script src="https://ajax.googleapis.com/ajax/libs/jquery/3.3.1/jquery.min.js"></script>
        <script type="text/javascript" src="https://unpkg.com/tabulator-tables@4.2.3/dist/js/tabulator.min.js"></script>
        </head>
        <body style="background-color:black;">

        <button id="get-selected">gen</button>
        <button id="select-all">all</button>
        <button id="deselect-all">none</button>
        <button id="clean-selected">clean</button>
        <div id="example-table"></div>

        <script>
        //define some sample data
        var tabledata = {table_data};
        </script>

        <script>
        var table = new Tabulator("#example-table", {{
            height: 600,
            selectable: true,
            data: tabledata,

            columns: [
                {{title: "lang", field: "lang_dir", sorter: "string", align: "right", visible: true, headerFilter: true}},
                {{title: "name", field: "proj_dir", sorter: "string", align: "left", visible: true, headerFilter: true}},
                {{title: "ver", field: "info.version", sorter: "string", align: "left", visible: true, headerFilter: true}}
            ],
        }});
        </script>

        <script>
        //select row on "select all" button click
        $("#select-all").click(function(){{
            table.selectRow();
        }});

        //deselect row on "deselect all" button click
        $("#deselect-all").click(function(){{
            table.deselectRow();
        }});

        

        $("#get-selected").click(function(){{
            var selected_data = table.getSelectedData();
            console.log(selected_data);
            $.ajax({{
                type: "POST",
                url: "gen_projs",
                data: JSON.stringify( {{ dirs: selected_data }} ),
                // data: {{ dirs: selected_data }},
                success: null,
                dataType: "json",
                contentType: "application/json"
            }});
        }});

        $("#clean-selected").click(function(){{
            var selected_data = table.getSelectedData();
            console.log(selected_data);
            $.ajax({{
                type: "POST",
                url: "clear_projs",
                data: JSON.stringify( {{ dirs: selected_data }} ),
                // data: {{ dirs: selected_data }},
                success: null,
                dataType: "json",
                contentType: "application/json"
            }});
        }});
        </script>

        </body>
        </html>
        "##,
        table_data = dirs,
    );

    aweb::HttpResponse::Ok().body(data)
}
