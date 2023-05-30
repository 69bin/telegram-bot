use rand::Rng;

pub fn generate_num() -> (i32,i32,i32){
    let mut rng = rand::thread_rng();
    let num1 = rng.gen_range(0..=100);
    let num2 = rng.gen_range(0..=100);
    let num3 = num1 + num2;
    (num1,num2,num3)
}


pub fn generate_10_num(num : i32) -> (i32,i32,i32,i32) {
    let mut rng = rand::thread_rng();
    let num1 = rng.gen_range(-8..=-2);
    let num2 = rng.gen_range(-2..=4);
    let num3 = rng.gen_range(4..=11);
    let num_location = rng.gen_range(0..=5);

    if num_location == 0{
        return (num,num+num1,num+num2,num+num3);
    }else if num_location == 1{
        return (num+num3,num,num+num1,num+num2);
    }else if num_location == 2{
        return (num+num2,num+num3,num,num+num1);
    }
    (num+num3,num+num2,num+num1,num)
}
