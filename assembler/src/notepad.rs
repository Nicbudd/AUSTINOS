let my_vec = vec![Some(1), Some(2), Some(3), Some(4), Some(5)]     vec: Some(1) Some(2) Some(3) Some(4) Some(5)
my_vec.place_arbitrary(Some(100), 7)                               vec: Some(1) Some(2) Some(3) Some(4) Some(5) None None Some(100)
