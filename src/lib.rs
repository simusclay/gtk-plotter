use std::cell::RefCell;
use std::cmp::Eq;
use std::collections::HashMap;
use std::rc::Rc;
use gtk::{prelude::*, DrawingArea};
use plotters_cairo::CairoBackend;

use plot_graph::liquidity_plot;
use plot_graph::{
    candles::plot,
    helpers,
    interactive_candles::launch_gui,
};

const GLADE_UI_SOURCE: &'static str = include_str!("ui.glade");
const TITLE: &str = "title"; //TODO
const FONT: &'static (&str,u32) = &("Montserrat", 14);
const CANDLE_SIZE_DIVIDER: f64 = 65.;

#[derive(PartialEq, Eq, Hash, Debug)]
enum GoodKind {
    USD,
    YEN,
    YUAN,
    EUR,
}

fn build_ui(app: &gtk::Application, data: &Rc<Vec<Vec<HashMap<GoodKind, Vec<f32>>>>>) {

    let liq = Rc::new(data_split(data));

    let builder = gtk::Builder::from_string(GLADE_UI_SOURCE);
    let window = builder.object::<gtk::Window>("MainWindow").unwrap();

    window.set_title(TITLE);

    // encapsulating all the objects
    let sell_usd: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("SellUSD").unwrap()));
    let sell_yen: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("SellYEN").unwrap()));
    let sell_yuan: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("SellYUAN").unwrap()));
    let buy_usd: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("BuyUSD").unwrap()));
    let buy_yen: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("BuyYEN").unwrap()));
    let buy_yuan: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("BuyYUAN").unwrap()));
    let liquidity: Rc<RefCell<gtk::DrawingArea>> =
        Rc::new(RefCell::new(builder.object("LiquidityPlot").unwrap()));

    let interactive_visualizer_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("Intbtn").unwrap()));

    let sell_usd_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("SellUSDbtn").unwrap()));
    let sell_yen_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("SellYENbtn").unwrap()));
    let sell_yuan_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("SellYUANbtn").unwrap()));
    let buy_usd_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("BuyUSDbtn").unwrap()));
    let buy_yen_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("BuyYENbtn").unwrap()));
    let buy_yuan_btn = Rc::new(RefCell::new(builder.object::<gtk::Button>("BuyYUANbtn").unwrap()));
    
    let start_state = Rc::new(RefCell::new(builder.object::<gtk::Scale>("StartingDayScale").unwrap()));
    let end_state = Rc::new(RefCell::new(builder.object::<gtk::Scale>("EndingDayScale").unwrap()));
    let market_state = Rc::new(RefCell::new(builder
        .object::<gtk::ComboBoxText>("MarketComboBox")
        .unwrap()));

    let yaxis = Rc::new(RefCell::new(builder.object::<gtk::Switch>("YAxis").unwrap()));
    let is_start_changing = Rc::new(RefCell::new(true));


    window.set_application(Some(app));
    

    // retieving all data
    

    // retrieving max len
    let max_len: f64 = data[0][0][&GoodKind::USD].len() as f64;

    // setting ranges of scales and initial states
    start_state.borrow_mut().set_range(0., max_len - 2.); 
    end_state.borrow_mut().set_range(2., max_len);
    end_state.borrow_mut().set_value(max_len);

    // let data = Rc::new(data);
    // let liq = Rc::new(liq);

    // binding buttons to their function
    btn_connect(data.clone(), 0, GoodKind::USD, &sell_usd_btn, &format!("Sell price USD"), &market_state,max_len);
    btn_connect(data.clone(), 0, GoodKind::YEN, &sell_yen_btn, &format!("Sell price YEN"), &market_state,max_len);
    btn_connect(data.clone(), 0, GoodKind::YUAN, &sell_yuan_btn, &format!("Sell price YUAN"), &market_state,max_len);
    btn_connect(data.clone(), 1, GoodKind::USD, &buy_usd_btn, &format!("Buy price USD"), &market_state,max_len);
    btn_connect(data.clone(), 1, GoodKind::YEN, &buy_yen_btn, &format!("Buy price YEN"), &market_state,max_len);
    btn_connect(data.clone(), 1, GoodKind::YUAN, &buy_yuan_btn, &format!("Buy price YUAN"), &market_state,max_len);

    // setting up the Interactive UI button
    let data_clone = data.clone();
    let market_state_clone = market_state.clone();
    interactive_visualizer_btn.borrow_mut().connect_clicked(move |_| {
        let market_index = market_state_clone.borrow().active().unwrap() as usize;
        let vec = vec![
            ("Sell price USD", data_clone[market_index][0][&GoodKind::USD].clone()),
            ("Sell price YEN", data_clone[market_index][0][&GoodKind::YEN].clone()),
            ("Sell price YUAN", data_clone[market_index][0][&GoodKind::YUAN].clone()),
            ("Buy price USD", data_clone[market_index][1][&GoodKind::USD].clone()),
            ("Buy price YEN", data_clone[market_index][1][&GoodKind::YEN].clone()),
            ("Buy price YUAN", data_clone[market_index][1][&GoodKind::YUAN].clone()),

        ];
        let mut candle_size = max_len/CANDLE_SIZE_DIVIDER;
        if candle_size <= 2. {
            candle_size = 2.;
        }
        launch_gui(vec,candle_size as usize).expect("interactive launch failed... ");
    });



    // todo func
    

    let (data_min,data_max) = max_data(&data);
    println!("- datamin {:?}  \n- datamax{:?}", data_min, data_max);

    let liq_max = max_liq(&liq);
    
    println!("- {:?}", liq_max);

    let data_min = Rc::new(data_min);
    let data_max = Rc::new(data_max);
    let liq_max = Rc::new(liq_max);

    // actually drawing the plots and binding them with all the objects they can interact with
    plot_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,data.clone(),0,GoodKind::USD,&sell_usd,&data_min,&data_max,);
    plot_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,data.clone(),0,GoodKind::YEN,&sell_yen,&data_min,&data_max,);
    plot_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,data.clone(),0,GoodKind::YUAN,&sell_yuan,&data_min,&data_max,);
    plot_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,data.clone(),1,GoodKind::USD,&buy_usd,&data_min,&data_max,);
    plot_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,data.clone(),1,GoodKind::YEN,&buy_yen,&data_min,&data_max,);
    plot_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,data.clone(),1,GoodKind::YUAN,&buy_yuan,&data_min,&data_max,);
    plot_liquidity_drawing_area(&yaxis,&start_state,&end_state,&market_state,&is_start_changing,liq.clone(),&liquidity,&liq_max,);

    window.show_all();

}

// function to connect buttons to their functionality
fn btn_connect(data: Rc<Vec<Vec<HashMap<GoodKind, Vec<f32>>>>>,
    op: usize,
    gk: GoodKind,
    btn: &Rc<RefCell<gtk::Button>>,    
    name: &String,
    market_state: &Rc<RefCell<gtk::ComboBoxText>>,
    max_len: f64,
) {
    let market_clone = market_state.clone();
    let name_clone = name.clone();
    
    // action on click
    btn.borrow_mut().connect_clicked(move |_| {
        let market_index = market_clone.borrow().active().unwrap() as usize;
        let mut candle_size = max_len/CANDLE_SIZE_DIVIDER;
        //  
        if candle_size <= 2. {
            candle_size = 2.;
        }
        launch_gui(vec![(name_clone.as_str(), data[market_index][op][&gk].clone())], candle_size as usize).expect("interactive launch failed... ");

    });
}

fn plot_drawing_area(
    yaxis: &Rc<RefCell<gtk::Switch>>,
    start_state: &Rc<RefCell<gtk::Scale>>,
    end_state: &Rc<RefCell<gtk::Scale>>,
    market_state: &Rc<RefCell<gtk::ComboBoxText>>,
    is_start_changing: &Rc<RefCell<bool>>,
    data: Rc<Vec<Vec<HashMap<GoodKind, Vec<f32>>>>>,
    // market_index: usize,
    op: usize,
    gk: GoodKind,
    draw_area: &Rc<RefCell<DrawingArea>>,
    data_min: &Rc<Vec<f32>>,
    data_max: &Rc<Vec<f32>>,
) {
    let start_cloned = start_state.clone();
    let end_cloned = end_state.clone();
    let is_start_changing_clone = is_start_changing.clone();
    let market_clone = market_state.clone();
    let data_min_clone = data_min.clone();
    let data_max_clone = data_max.clone();
    let yaxis_clone = yaxis.clone();

    draw_area.borrow().connect_draw(move |widget, cr| {
        let sw = yaxis_clone.borrow().is_active();
        let mut start = start_cloned.borrow().value() as usize;
        let mut end = end_cloned.borrow().value() as usize;
        let market_index = market_clone.borrow().active().unwrap() as usize;
        if start + 1 >= end {
            if *is_start_changing_clone.borrow() {
                end_cloned.borrow_mut().set_value(start as f64 + 2.);
                end = start + 2;
            } else {
                start_cloned.borrow_mut().set_value(end as f64 - 2.);
                start = end - 2;
            }
        }

        let w = widget.allocated_width();
        let h = widget.allocated_height();
        let backend = CairoBackend::new(cr, (w as u32, h as u32)).unwrap();


        let mut candle_size = (end-start)/(CANDLE_SIZE_DIVIDER as usize);
        if candle_size <= 2 {
            candle_size = 2;
        }


        let yaxis;
        if !sw {yaxis = None;}
        else {yaxis = Some((data_min_clone[market_index], data_max_clone[market_index]));}

        let op_string: &str;
        if op == 0 {op_string = "Sell"}
        else {op_string = "Buy"}

        plot(
            &data[market_index][op][&gk].clone()[start..end],
            candle_size,
            format!("{} {:?}",&op_string, &gk).as_str(),
            backend,
            40,
            400,
            *FONT,
            Some(start),
            yaxis,
        )
        .unwrap();
        Inhibit(false)
    });

    let draw_area_clone = draw_area.clone();
    let is_start_changing_clone = is_start_changing.clone();

    start_state.borrow_mut().connect_value_changed(move |_| {
        *is_start_changing_clone.borrow_mut() = true;

        draw_area_clone.borrow().queue_draw();
    });

    let draw_area_clone = draw_area.clone();
    let is_start_changing_clone = is_start_changing.clone();

    end_state.borrow_mut().connect_value_changed(move |_| {
        *is_start_changing_clone.borrow_mut() = false;

        draw_area_clone.borrow().queue_draw();
    });

    let draw_area_clone = draw_area.clone();
    yaxis.borrow_mut().connect_changed_active(move |_| {
        draw_area_clone.borrow().queue_draw();
    });

    let draw_area_clone = draw_area.clone();
    market_state.borrow_mut().connect_changed(move |_| {
        draw_area_clone.borrow().queue_draw();
    });




}

fn gtk_plotter(data: Vec<Vec<HashMap<GoodKind, Vec<f32>>>>) {

    // let mut data: Vec<Vec<HashMap<GoodKind, Vec<f32>>>> = Vec::new();
    // // let mut liq: Vec<Vec<Vec<f32>>> = vec![
    // //     // bose
    // //     vec![
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //     ],
    // //     // bfb
    // //     vec![
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //     ],
    // //     vec![
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //         generate_data_series(100., 2000, -0.0985, 0.1),
    // //     ],
    // // ];

    // data.push(vec![HashMap::new(), HashMap::new(), HashMap::new()]);
    // // sell
    // // data.get_mut(0)
    // //     .unwrap()
    // //     .get_mut(0)
    // //     .unwrap()
    // //     .insert(GoodKind::EUR, generate_data_series(100., 200, -0.0985, 0.1));
    // data.get_mut(0)
    //     .unwrap()
    //     .get_mut(0)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(1.1, 2000, -0.0985, 0.1));
    // data.get_mut(0)
    //     .unwrap()
    //     .get_mut(0)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(0.8, 2000, -0.0985, 0.1));
    // data.get_mut(0).unwrap().get_mut(0).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(0.5, 2000, -0.0985, 0.1),
    // ); //buy
    //    // data.get_mut(0)
    //    //     .unwrap()
    //    //     .get_mut(1)
    //    //     .unwrap()
    //    //     .insert(GoodKind::EUR, generate_data_series(100., 200, -0.0985, 0.1));
    // data.get_mut(0)
    //     .unwrap()
    //     .get_mut(1)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(0)
    //     .unwrap()
    //     .get_mut(1)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(0).unwrap().get_mut(1).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );

    // data.get_mut(0)
    //     .unwrap()
    //     .get_mut(2)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(0)
    //     .unwrap()
    //     .get_mut(2)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(0).unwrap().get_mut(2).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );
    // data.get_mut(0).unwrap().get_mut(2).unwrap().insert(
    //     GoodKind::EUR,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );

    // //other market
    // //sell
    // data.push(vec![HashMap::new(), HashMap::new(), HashMap::new()]);
    // // data.get_mut(1)
    // //     .unwrap()
    // //     .get_mut(0)
    // //     .unwrap()
    // //     .insert(GoodKind::EUR, generate_data_series(100., 200, -0.0985, 0.1));
    // data.get_mut(1)
    //     .unwrap()
    //     .get_mut(0)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(1)
    //     .unwrap()
    //     .get_mut(0)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(1).unwrap().get_mut(0).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // ); // buy
    // // data.get_mut(1)
    // //     .unwrap()
    // //     .get_mut(1)
    // //     .unwrap()
    // //     .insert(GoodKind::EUR, generate_data_series(100., 200, -0.0985, 0.1));
    // data.get_mut(1)
    //     .unwrap()
    //     .get_mut(1)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(1)
    //     .unwrap()
    //     .get_mut(1)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(1).unwrap().get_mut(1).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );
    // data.get_mut(1)
    //     .unwrap()
    //     .get_mut(2)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(1)
    //     .unwrap()
    //     .get_mut(2)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(1).unwrap().get_mut(2).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );
    // data.get_mut(1).unwrap().get_mut(2).unwrap().insert(
    //     GoodKind::EUR,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );

    // data.push(vec![HashMap::new(), HashMap::new(), HashMap::new()]);

    // data.get_mut(2)
    //     .unwrap()
    //     .get_mut(0)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(2)
    //     .unwrap()
    //     .get_mut(0)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(2).unwrap().get_mut(0).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // ); // buy
    // // data.get_mut(1)
    // //     .unwrap()
    // //     .get_mut(1)
    // //     .unwrap()
    // //     .insert(GoodKind::EUR, generate_data_series(100., 200, -0.0985, 0.1));
    // data.get_mut(2)
    //     .unwrap()
    //     .get_mut(1)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(2)
    //     .unwrap()
    //     .get_mut(1)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(2).unwrap().get_mut(1).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );
    // data.get_mut(2)
    //     .unwrap()
    //     .get_mut(2)
    //     .unwrap()
    //     .insert(GoodKind::USD, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(2)
    //     .unwrap()
    //     .get_mut(2)
    //     .unwrap()
    //     .insert(GoodKind::YEN, generate_data_series(100., 2000, -0.0985, 0.1));
    // data.get_mut(2).unwrap().get_mut(2).unwrap().insert(
    //     GoodKind::YUAN,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );
    // data.get_mut(2).unwrap().get_mut(2).unwrap().insert(
    //     GoodKind::EUR,
    //     generate_data_series(100., 2000, -0.0985, 0.1),
    // );

    let data = Rc::new(data);
    // let liq = Rc::new(liq);




    let application = gtk::Application::new(
        Some("com.example"), // TODO
        Default::default(),
    );

    // let data_clone = data.clone();
    // let liq_clone = liq.clone();
    application.connect_activate(move |app| {
        build_ui(app,&data);
    });

    application.run();
}

fn plot_liquidity_drawing_area(
    yaxis: &Rc<RefCell<gtk::Switch>>,
    start_state: &Rc<RefCell<gtk::Scale>>,
    end_state: &Rc<RefCell<gtk::Scale>>,
    market_state: &Rc<RefCell<gtk::ComboBoxText>>,
    is_start_changing: &Rc<RefCell<bool>>,
    liq: Rc<Vec<Vec<Vec<f32>>>>,
    // market_index: usize,
    draw_area: &Rc<RefCell<DrawingArea>>,
    liq_max: &Rc<Vec<f32>>,
) {
    let start_cloned = start_state.clone();
    let end_cloned = end_state.clone();
    let is_start_changing_clone = is_start_changing.clone();
    let market_clone = market_state.clone();
    let liq_max_clone = liq_max.clone();
    let yaxis_clone = yaxis.clone();

    draw_area.borrow().connect_draw(move |widget, cr| {
        let sw = yaxis_clone.borrow().is_active();
        let mut start = start_cloned.borrow().value() as usize;
        let mut end = end_cloned.borrow().value() as usize;
        let market_index = market_clone.borrow().active().unwrap() as usize;

        if start + 1 >= end {
            if *is_start_changing_clone.borrow() {
                end_cloned.borrow_mut().set_value(start as f64 + 2.);
                end = start + 2;
            } else {
                start_cloned.borrow_mut().set_value(end as f64 - 2.);
                start = end - 2;
            }
        }

        let yaxis;
        if !sw {yaxis = None;}
        else {yaxis = Some((0., liq_max_clone[market_index]));}

        let w = widget.allocated_width();
        let h = widget.allocated_height();
        let backend = CairoBackend::new(cr, (w as u32, h as u32)).unwrap();
        liquidity_plot::plot(
            vec![
                &liq[market_index][0].clone()[start..end],
                &liq[market_index][1].clone()[start..end],
                &liq[market_index][2].clone()[start..end],
                &liq[market_index][3].clone()[start..end],
            ],
            vec![
                format!("USD"),
                format!("YEN"),
                format!("YUAN"),
                format!("EUR"),
            ],
            backend,
            Some(start),
            yaxis,
            10,
            16,
            *FONT,
        )
        .unwrap();
        Inhibit(false)
    });

    let draw_area_clone = draw_area.clone();
    let is_start_changing_clone = is_start_changing.clone();

    start_state.borrow_mut().connect_value_changed(move |_| {
        *is_start_changing_clone.borrow_mut() = true;
        draw_area_clone.borrow().queue_draw();
    });

    let draw_area_clone = draw_area.clone();
    let is_start_changing_clone = is_start_changing.clone();

    end_state.borrow_mut().connect_value_changed(move |_| {
        *is_start_changing_clone.borrow_mut() = false;
        draw_area_clone.borrow().queue_draw();
    });

    let draw_area_clone = draw_area.clone();
    yaxis.borrow_mut().connect_changed_active(move |_| {
        draw_area_clone.borrow().queue_draw();
    });

    let draw_area_clone = draw_area.clone();

    market_state.borrow_mut().connect_changed(move |_| {
        draw_area_clone.borrow().queue_draw();
    });
}

/*

    let mut data: Vec<Vec<HashMap<GoodKind, Vec<f32>>>>

        data ->

            0 (bose) ->

                0 (buy) ->
                    vec<f32>

                1 (sell) ->
                    vec<f32>

            1 (bfb) ->  ...
            2 (doge) -> ...


    let liq: Vec<Vec<Vec<f32>>>

        liq ->

            0 (bose) ->
                0(EUR) ->
                    vec<f32>
                1(USD)

*/

#[allow(unused_assignments)]
fn max_data(data: &Rc<Vec<Vec<HashMap<GoodKind, Vec<f32>>>>>) -> (Vec<f32>,Vec<f32>) {

    let mut curr_max_y: f32 = 0.;
    let mut curr_min_y: f32 = f32::MAX;
    let v: Vec<GoodKind> = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    let mut data_min: Vec<f32> = vec![f32::MAX,f32::MAX,f32::MAX];
    let mut data_max: Vec<f32> = vec![0.,0.,0.];
    for market in 0..3 { //market

        for op in 0..2 { // sell -> usd
            
            for gk in v.iter(){ // goodkind
                
                curr_min_y = helpers::f32_min(&data[market][op][gk]);
                curr_max_y = helpers::f32_max(&data[market][op][gk]);
                if data_max[market] <= curr_max_y {
                    // println!("{}-{}-{:?}", i,j,k);
                    data_max[market] = curr_max_y;
                }
                if data_min[market] >= curr_min_y {
                    data_min[market] = curr_min_y;
                }
            }
            
        }
    }
    (data_min,data_max)
}
#[allow(unused_assignments)]

fn max_liq(liq: &Rc<Vec<Vec<Vec<f32>>>>) -> Vec<f32> {
    let mut curr_max_y = 0.;
    let mut liq_max: Vec<f32> = vec![0.,0.,0.];
    for market in 0..3 { // market
        for gk in 0..4 { // goodkind
            curr_max_y = helpers::f32_max(&liq[market][gk]);
            if liq_max[market] <= curr_max_y {
                liq_max[market] = curr_max_y;
            }
        }
    }
    liq_max
}




fn data_split(data: &Rc<Vec<Vec<HashMap<GoodKind, Vec<f32>>>>>) -> Vec<Vec<Vec<f32>>>{
    let mut liq: Vec<Vec<Vec<f32>>> = vec![vec![Vec::new();4];3];
    let op: usize = 2;
    for (i,market) in data.iter().enumerate() {
        for (gk,v) in  market[2].iter(){
            match gk {
                GoodKind::USD => {liq[i][0] = v.clone()},
                GoodKind::YEN => {liq[i][1] = v.clone()},
                GoodKind::YUAN => {liq[i][2] = v.clone()},
                GoodKind::EUR => {liq[i][3] = v.clone()},
            }
        }
    }
    liq
}