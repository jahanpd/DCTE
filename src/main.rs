use yew::prelude::*;
use yew::{Properties};
use agesim::{Settings, Organism};
use wasm_logger;
use log;
use gloo_timers::callback::Timeout;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend; 
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::f64;
use colorgrad;

mod styles;
use styles::{Themes};
use stylist::yew::{styled_component, Global};


#[derive(Clone, PartialEq, Properties)]
struct OrganismProps {
    settings: Settings,
    organism: Organism,
    sizes: Vec<f32>,
    ages: Vec<f32>
}

fn drawsim(org: Organism) {
    let maxage: f64 = org.ages.clone().into_iter().reduce(f32::min).unwrap_or(0f32) as f64;
    let grad = colorgrad::CustomGradient::new()
        .html_colors(&["deeppink", "gold", "seagreen"])
        .build().unwrap();
    let window = web_sys::window().unwrap();
    let view_width = window.inner_width().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("simulation").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    let width = (view_width.as_f64().unwrap() * 0.6).min(600.0);
    let height = (view_width.as_f64().unwrap() * 0.6).min(600.0);
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    assert_eq!(width, height);
    let diameter = (width as f64) / (org.settings.length as f64);
    context.begin_path();
    for (i, coords) in org.coordinates.iter().enumerate() {
        let relage = (org.ages[i] as f64) / 150f64.max(maxage);
        let colour = grad.at(relage).to_hex_string();
        context
            .set_fill_style(&JsValue::from_str(&colour));
        context
            .fill_rect(
                (coords.x as f64) * diameter, 
                (coords.y as f64) * diameter,
                diameter, diameter
            )
    }
    // Draw the outer circle.
    context.stroke();
}

// plotting function
fn drawplot(canvas_id: &str, age: Vec<f32>, size: Vec<f32>) {
    let window = web_sys::window().unwrap();
    let view_width = window.inner_width().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("dataplot").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let width = (view_width.as_f64().unwrap() * 0.8).min(600.0);
    let height = (view_width.as_f64().unwrap() * (8.0 / 15.0)).min(400.0);
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
    let root = backend.into_drawing_area();
    root.fill(&WHITE).unwrap();
    // After this point, we should be able to draw construct a chart context
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("Mean Age and Organism Size over Time", ("sans-serif", 14).into_font())
        // Set the size of the label region
        .x_label_area_size(50u32)
        .margin(5u32)
        .right_y_label_area_size(60u32)
        .y_label_area_size(60u32)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(0f32..(age.len() as f32), 0f32..(age.clone().into_iter().reduce(f32::max).unwrap_or(1f32) as f32)).unwrap()
        .set_secondary_coord(0f32..(size.len() as f32), 0f32..(size.clone().into_iter().reduce(f32::max).unwrap_or(1f32) + 10f32 as f32));
    // And we can draw something in the drawing area
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_desc("Age")
        .y_label_formatter(&|x| format!("{:e}", x))
        .draw().unwrap();

    chart
        .configure_secondary_axes()
        .y_desc("Size")
        .draw().unwrap();

    chart.draw_series(LineSeries::new(
        age.clone().iter().enumerate().map(|(i, x)| (i as f32, *x as f32)).collect::<Vec<(f32, f32)>>(),
        &RED,
    )).unwrap();

    chart.draw_secondary_series(LineSeries::new(
        size.clone().iter().enumerate().map(|(i, x)| (i as f32, *x as f32)).collect::<Vec<(f32, f32)>>(),
        &BLUE,
    )).unwrap();

    root.present().unwrap();
}

#[styled_component(RenderOrganism)]
fn render_organism(orgpoint: &OrganismProps) -> Html{
    let orgprops = use_state(|| orgpoint.clone());
    let step = use_state(|| 0);
    let grow = use_state(|| false);

    let settings = orgprops.settings.clone();
    let age = orgprops.organism.mean_age();
    let size = orgprops.organism.size;
    let ages = orgprops.ages.clone();
    let sizes = orgprops.sizes.clone();
    let entropy = orgprops.organism.entropy();

    let organism = orgprops.organism.clone();
    let exceed = *step < 10000;
    
    if *grow & exceed {
        let timeout = Timeout::new(15, move || {
            let mut sizes = orgprops.sizes.clone();
            sizes.push(size as f32);
            let mut ages = orgprops.ages.clone();
            ages.push(age);
            let neworgprops = OrganismProps {
                settings: orgprops.settings.clone(),
                organism: orgprops.organism.clone().growstep(),
                sizes: sizes,
                ages: ages
            };
            let counter = *step + 1;
            step.set(counter);
            orgprops.set(neworgprops)
            }
        );
        timeout.forget();
    };

    let toggle_start = {
        let grow = grow.clone();
        move |_| {
            let value = !*grow;
            grow.set(value);
        }
    };


    // render age plot
    drawplot("dataplot", ages, sizes);
    // render simulation
    drawsim(organism);
    log::info!("Update: {:?}", exceed);
    html! {
    <>
        <div class="flex justify-center flex-wrap flex-col items-center">
            if *grow & exceed {
                <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded w-4/12" onclick={toggle_start}>{ "Pause Simulation" }</button>
            } else {
                if !exceed {
                <button class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded w-4/12">{ "Reload Page to Restart" }</button>
                } else {
                <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded w-4/12" onclick={toggle_start}>{ "Start Simulation" }</button>
                }
            }
            <p class="sm:w-8/12 w-full text-center text-lg pt-8"> <b>{ "Entropy of each Base" } </b></p>
            <p class="sm:w-8/12 w-full text-left pb-5"> { "\
                Each cell has a simple genome represented as a string, in this case GATTACA. \
                At the beginning, each base of the genome has an information entropy of 0 as all the early cells have the same genome. \
                As cells accumulate mutations over time and with cell division, the entropy of each base increases. \
                See below for more details. \
                " } </p>
            {for entropy.into_iter().map(|el| html!{<p> {format!("{}",el.0)} {":"} {format!("{}", el.1)} </p>})}
        </div>
    </>
    }
}


#[styled_component(AppComponent)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());
    let settings: Settings = Settings {
            length: 20,
            genome: "GATTACA".to_string(),
            mutation_rate: 0.00016,
            growth_rate: 0.01,
            seed: 1234
        };

    let organism: Organism = settings.init_organism();
    let orgprops = OrganismProps {
        settings: settings,
        organism: organism,
        sizes: vec![],
        ages: vec![]
    };
    let themes: Themes = Themes{};
    html! {
<>
    <div class="flex flex-row flex-wrap h-screen w-screen">
        <div class="flex flex-row flex-wrap pt-2 pb-2 pl-5 pr-5 w-full justify-around">
            <h1 class="w-full text-center text-xl p-1"><b>{"Decentralized cellular timekeeping based on entropy"}</b></h1>
            <h2 class= "w-full text-center p-1">{"A mechanism for groups of cells to estimate the age of the organism they form."}</h2>
            <h2 class= "w-full text-center text-sm p-1 italic">{"J. C. Penny-Dimri"}</h2>
        </div>
        <div class="flex flex-col items-center w-full justify-around pt-2 pb-2 pl-5 pr-5">
            <canvas id="simulation"  width="600" height="600"></canvas>
            <p class="sm:w-3/12 w-full text-center text-sm pb-5"> {"Cells that estimate organismal age as young are pink, and turn green as they determine the organism is older."} </p>
        </div>
        <div class="flex flex-row w-full justify-around pt-2 pb-2 pl-5 pr-5">
            <RenderOrganism ..orgprops/>
        </div>
        <div class="flex flex-row w-full justify-around pt-2 pb-2 pl-5 pr-5">
            <canvas id="dataplot" width="600" height="400"></canvas>
        </div>
        <div class="flex flex-col w-full items-center pt-2 pb-2 pl-5 pr-5">
            <p class="sm:w-8/12 w-full text-center text-lg pt-8"> <b>{ "Rules of the Model" } </b></p>
            <p class="sm:w-8/12 w-full text-left pb-2"> { "\
                The high level explanation of the model is that at each time point cells are comparing their genomes with their neighbours and 'measuring' the differences between them. \
                The greater the number of differences, or 'entropy' in terms of information theory, the greater the age of the organism. \
                Such a method of measuring could then be used to regulate development and affect cellular function with age. \
                This will become clearer when we consider the rules of the model and how this mechanism might look in a real organism. \
                " } </p> <br/>
            <p class="sm:w-8/12 w-full text-left pb-3"> { "Rules:" }</p>
            <ol class="list-decimal text-left sm:w-8/12 w-full">
                <li> {"Initialize the first cell with a genome of string GATTACA."} </li>
                <li> {"At each timestep there is a probability of accumulating mutations."} </li>
                <li> {"At each timestep there is a probability of splitting that decreases exponentially with a cells estimate of age."} </li>
                <li> {"Each split incurrs a change of accumulating mutations."} </li>
                <li> {"At each timestep a cell has a probability of recieving a message from a neigbour proportional to the distance from the neighbour."} </li>
                <li> {"The message is a representation of the neighbouring cells genome, eg RNA in an extracellular vescicle."} </li>
                <li> {"The cells compare their own genome to the messages of their neighbours and express an age based on the number of differences detected."} </li>
            </ol>
            <p class="sm:w-8/12 w-full text-center text-lg pt-8"> <b>{ "Theoretical Biological System" } </b></p>
            <p class="sm:w-8/12 w-full text-left pb-2"> { "\
                What is presented above is a computational model that demonstrates how such a system could regulate growth and cellular function. \
                Critically, this is acheived without a centralized timekeeping authority and relies only on gene mututations and message passing. \
                Therefore, it is a highly plausible mechanism for real biological systems. \
                " } </p>
            <p class="sm:w-8/12 w-full text-left pb-2"> { "\
                An interesting candidate system could be RNA fragments contained in extracellular vesicles (ECVs). \
                RNA is more or less a direct representation of a cells DNA that can be packaged up into ECVs and transported to neighbouring (and sometimes distant) cells. \
                With this in mind, we could envisage that some protein encoding gene has a sense RNA, and its antisense counterpart transcribed. \
                We could then imagine that the antisense fragments are packaged up into ECVs and transported to neighbours where they act to suppress the expression of the gene. \
                As each cell accumulates errors in these genes, the antisense fragments become worse at suppressing expression in their neighbours. \
                The cell can then 'measure' age by the expression of our hypothetical gene increasing. \
                " } </p>
            <p class="sm:w-8/12 w-full text-left pb-2"> { "\
                ECVs, however, carry a complex bundle of RNA fragements. \
                Another more likely interpretation could be that these packets of information help tell neighbouring cells what phenotype they should be expressing. \
                For example, a dermal fibroblast surrounded by skin cells in a young environment can be quite confident it should be a dermal fibroblast. \
                As surrounding cells begin to accumulate mutations, entropy (uncertainty) increases, and these packets of information become less clear. \
                The aforementioned dermal fibroblast then becomes less confident in it's phenotype and appears to 'age' at a cellular level.
                " } </p>
            <p class="sm:w-8/12 w-full text-center text-lg pt-8"> <b>{ "Why is this Important" } </b></p>
            <p class="sm:w-8/12 w-full text-left pb-2"> { "\
                There are two separate dichotomies in understanding ageing that remain problematic. \
                Firstly, there has yet to be a theoretical or experimental model that unifies cellular and organismal aging. \
                Secondly, it is unclear how much of ageing is due to an underlying program or caused by 'wear and tear' (entropic decline).\
                This model presents an opportunity provide a link between entropic decline and developemental or programmed changes \
                as well as a better understanding of how changes to a cell can affect tissues and the organism.
                " } </p>
          <p class="sm:w-8/12 w-full text-left pb-2"> { "\
                Furthermore, if this model represents a true mechanism of ageing then this would fundamentally change future efforts to engineer therapeutics. \
                For example, epigenitic reprogramming would likely be ineffective in the long run as it \
                does not impact the underlying stochastic mutations that are proposed to underpin the aged phenotype. \
                " } </p>

        </div>
    </div>
</>
    }
}

fn main() {
    yew::Renderer::<AppComponent>::new().render();
    // yew::start_app::<AppComponent>();
}
