//!
//! Wraps a dinotree and measure the time of rebalancing and collision finding over multiple world steps. It can then produce a graph using rust gnu plot.
//!

extern crate dinotree;
extern crate dinotree_alg;
extern crate gnuplot;
extern crate axgeom;
use axgeom::*;
use dinotree::*;

use std::time::Instant;

use gnuplot::*;



pub struct DinoTreeCache<A:AxisTrait,T:HasAabb>{
	axis:A,
	a:Option<DinoTree<A,(),T>>,
	counter:bool
}

impl<A:AxisTrait,T:Copy,Num:NumTrait> DinoTreeCache<A,BBox<Num,T>>{
	pub fn new(axis:A)->DinoTreeCache<A,BBox<Num,T>>{
		DinoTreeCache{a:None,axis,counter:true}
	}

	pub fn get_tree_normal(&mut self,bots:&[T],func:impl FnMut(&T)->axgeom::Rect<Num>)->&mut DinoTree<A,(),BBox<Num,T>>{
		self.a=Some(DinoTree::new(self.axis,(),bots,func));
		return self.a.as_mut().unwrap()
	}
	pub fn get_tree(&mut self,bots:&[T],func:impl FnMut(&T)->axgeom::Rect<Num>)->&mut DinoTree<A,(),BBox<Num,T>>{
		if self.a.is_none(){
			assert_eq!(self.counter,true);
			let tree=DinoTree::new(self.axis,(),bots,func);
			self.a=Some(tree);
			self.counter=false;
			return self.a.as_mut().unwrap();	
		}

		if self.counter == true{
			self.a=Some(DinoTree::new(self.axis,(),bots,func));
			//println!("a");
		}else{
			self.a.as_mut().unwrap().apply_into(bots,|a,b|b.inner=*a);
			//Need to apply bots here
			//println!("b");
		}

		self.counter=!self.counter;

		return self.a.as_mut().unwrap();
	}
}



fn into_secs(elapsed:std::time::Duration)->f64{
    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    sec
}

struct Round{
	rebal:f64,
	query:f64
}

pub struct Session{
	//tree:DinoTreeCache<axgeom::YAXISS,T:HasAabb>
	//tree:DinoTree<axgeom::YAXISS,T>
	times:Vec<Round>
}

impl Session{
	pub fn new()->Session{
		Session{times:Vec::new()}
	}

	pub fn finish(&self){
		//TODO draw graph here!!!

		let mut fg = Figure::new();


		let x=self.times.iter().enumerate().map(|(i,_)|i);
		let y1=self.times.iter().map(|a|a.rebal);
		let y2=self.times.iter().map(|a|a.query);

		fg.axes2d()
		.lines(x.clone(), y1, &[Caption("Rebal"), LineWidth(3.0), Color("red")])
		.lines(x, y2, &[Caption("Query"), LineWidth(3.0), Color("blue")]);
		
		fg.show();
		/*
	    fg.axis2d()	
	    fn draw_graph(title_name:&str,fg:&mut Figure,res:&Vec<TheoryRes>,rebal:bool,pos:usize){

	    	let ax=fg.axes2d().set_pos_grid(2,1,pos as u32)
		        .set_title(title_name, &[])
		        .set_x_label("Spiral Grow", &[])
		        .set_y_label("Number of Comparisons", &[]);
		  
		  	let num=res.first().unwrap().rebal.len();


		  	let x=res.iter().map(|a|a.grow);
	    
	    	if rebal{
		  		let cc=(0..num).map(|ii:usize|{res.iter().map(move |a|a.rebal[ii])});

			  	for (i,(col,y)) in cols.iter().cycle().zip( cc   ).enumerate(){
			  		let s=format!("Level {}",i);
			  		ax.lines(x.clone(),y,&[Color(col),Caption(&s),LineWidth(2.0)]);
			  	}
			}else{
				let cc=(0..num).map(|ii:usize|{res.iter().map(move |a|a.query[ii])});
				
			  	for (i,(col,y)) in cols.iter().cycle().zip( cc   ).enumerate(){
			  		let s=format!("Level {}",i);
			  		ax.lines(x.clone(),y,&[Color(col),Caption(&s),LineWidth(2.0)]);
			  	}
			}
		}
		*/

	}
}


pub struct DinoTreeMeasure<A:AxisTrait,T:HasAabb>{
	tree:DinoTree<A,(),T>,
	//leveltimer:Option<Vec<f64>>
	time:Option<f64>
}

impl<A:AxisTrait,T:Copy,N:NumTrait> DinoTreeMeasure<A,BBox<N,T>>{

	pub fn new(axis:A,bots:&[T],func:impl FnMut(&T)->axgeom::Rect<N>)->DinoTreeMeasure<A,BBox<N,T>>{
		/*
		let leveltimer=dinotree::advanced::LevelTimer::new();

		let a=dinotree::advanced::compute_tree_height_heuristic(bots.len());
		let k=dinotree::advanced::compute_default_level_switch_sequential();
		let (tree,leveltimer)=dinotree::advanced::new_adv(axis,(),bots,func,a,leveltimer,k);
		DinoTreeMeasure{tree,leveltimer:Some(leveltimer.into_inner())}
		
		*/
		let instant=Instant::now();

		let tree=DinoTree::new(axis,(),bots,func);
		let time=into_secs(instant.elapsed());

		DinoTreeMeasure{tree,time:Some(time)}
	}
}

impl<A:AxisTrait,T:HasAabb+Send> DinoTreeMeasure<A,T>{

	pub fn get_inner(&mut self)->&mut DinoTree<A,(),T>{
		&mut self.tree
	}
	pub fn query_mut(&mut self,session:&mut Session,func:impl Fn(&mut T,&mut T)+Copy+Clone+Send){
		let instant=Instant::now();
		dinotree_alg::colfind::query_mut(self.tree.as_ref_mut(),func);
		let time=into_secs(instant.elapsed());
		session.times.push(Round{rebal:self.time.take().unwrap(),query:time});
		/*
		use std::marker::PhantomData;
		use dinotree_alg::colfind::ColMulti;
		use dinotree::advanced::Splitter;
		
		struct Bo<T,F>(F,PhantomData<T>);

	    impl<T:HasAabb,F:Fn(&mut T,&mut T)> ColMulti for Bo<T,F>{
	        type T=T;
	        fn collide(&mut self,a:&mut T,b:&mut T){
	            self.0(a,b);
	        }   
	    }

	    impl<T,F:Copy> Splitter for Bo<T,F>{
	        fn div(self)->(Self,Self){
	        	let a=Bo(self.0,PhantomData);
	        	(self,a)
	        }
	        fn add(self,_:Self)->Self{
	            self
	        }
	        fn node_start(&mut self){}
	        fn node_end(&mut self){}
	    }


		let k=dinotree::advanced::compute_default_level_switch_sequential();
		let leveltimer2=dinotree::advanced::LevelTimer::new();
		let (_bo,leveltimer2)=dinotree_alg::colfind::query_adv_mut(&mut self.tree,Bo(func,PhantomData),leveltimer2,k);

		session.times.push(Round{rebal:self.leveltimer.take().unwrap(),query:leveltimer2.into_inner()})
		*/
	}
}
