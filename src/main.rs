use std::any::type_name;
use std::marker::PhantomData;

enum Uninhabited {}

struct Zero(Uninhabited);
struct Succ<I>(PhantomData<I>, Uninhabited);

trait TNumber {
    type Incr: TNumber;
    type Decr: TNumber;
}

impl TNumber for Zero {
    type Incr = Succ<Zero>;
    type Decr = Zero;
}

impl<I: TNumber> TNumber for Succ<I> {
    type Incr = Succ<Self>;
    type Decr = I;
}

struct Nil(Uninhabited);
struct Cons<Elem, Xs>(PhantomData<Elem>, PhantomData<Xs>, Uninhabited);

trait TInfStack<D> {
    type Head;
    type Tail: TInfStack<D>;
    type Set<Val>: TInfStack<D>;
}

impl<Elem, D, Xs: TInfStack<D>> TInfStack<D> for Cons<Elem, Xs> {
    type Head = Elem;
    type Tail = Xs;
    type Set<Val> = Cons<Val, Xs>;
}

impl<D> TInfStack<D> for Nil {
    type Head = D;
    type Tail = Cons<D, Nil>;
    type Set<Val> = Cons<Val, Nil>;
}

struct TDoubleStack<Left, Right>(PhantomData<Left>, PhantomData<Right>, Uninhabited);

trait InfTape<D> {
    type MoveRight: InfTape<D>;
    type MoveLeft: InfTape<D>;
    type Read;
    type Write<Val>: InfTape<D>;
}

impl<Left, Right, D> InfTape<D> for TDoubleStack<Left, Right>
where
    Left: TInfStack<D>,
    Right: TInfStack<D>,
{
    type MoveRight = TDoubleStack<Cons<<Right as TInfStack<D>>::Head, Left>, Right::Tail>;
    type MoveLeft =
        TDoubleStack<<Left as TInfStack<D>>::Tail, Cons<<Left as TInfStack<D>>::Head, Right>>;
    type Read = <Right as TInfStack<D>>::Head;
    type Write<Val> = TDoubleStack<Left, <Right as TInfStack<D>>::Set<Val>>;
}

struct MoveRight(Uninhabited);
struct MoveLeft(Uninhabited);
struct Increment(Uninhabited);
struct Decrement(Uninhabited);
struct EndLoop(Uninhabited);
struct StartLoop(Uninhabited);
struct Halt(Uninhabited);

struct FetchGoingBack<Depth>(PhantomData<Depth>, Uninhabited);
struct GoingBack<Saw, Depth>(PhantomData<Saw>, PhantomData<Depth>, Uninhabited);
struct Execute<Inst>(PhantomData<Inst>, Uninhabited);
struct Fetch(Uninhabited);
struct LoopCompare<Num>(PhantomData<Num>, Uninhabited);

struct InterpreterState<Code: InfTape<Halt>, Mem: InfTape<Zero>, State>(
    PhantomData<Code>,
    PhantomData<Mem>,
    PhantomData<State>,
    Uninhabited,
);

trait Interpreter {
    type FinalState;
}

type IRec<I: Interpreter> = <I as Interpreter>::FinalState;

// Fetch
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter for InterpreterState<Code, Mem, Fetch>
where
    InterpreterState<Code, Mem, Execute<Code::Read>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code, Mem, Execute<Code::Read>>>;
}

// Halt
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<Halt>>
{
    type FinalState = Self;
}

// Move right
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<MoveRight>>
where
    InterpreterState<Code::MoveRight, Mem::MoveRight, Fetch>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveRight, Mem::MoveRight, Fetch>>;
}

// Move left
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<MoveLeft>>
where
    InterpreterState<Code::MoveRight, Mem::MoveLeft, Fetch>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveRight, Mem::MoveLeft, Fetch>>;
}

// Increment
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<Increment>>
where
    Mem::Read: TNumber,
    InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Incr>, Fetch>: Interpreter,
{
    type FinalState =
        IRec<InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Incr>, Fetch>>;
}

// Decrement
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<Decrement>>
where
    Mem::Read: TNumber,
    InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Decr>, Fetch>: Interpreter,
{
    type FinalState =
        IRec<InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Decr>, Fetch>>;
}

// Execute startloop (no-op)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<StartLoop>>
where
    InterpreterState<Code::MoveRight, Mem, Fetch>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveRight, Mem, Fetch>>;
}

// Execute endloop (begin compare)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, Execute<EndLoop>>
where
    InterpreterState<Code, Mem, LoopCompare<Mem::Read>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code, Mem, LoopCompare<Mem::Read>>>;
}

// Loop value != 0 (begin loop)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, LoopCompare<Succ<I>>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<Zero>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<Zero>>>;
}

// Loop value == 0 (no-op)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero, Read = Zero>> Interpreter
    for InterpreterState<Code, Mem, LoopCompare<Zero>>
where
    InterpreterState<Code::MoveRight, Mem, Fetch>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveRight, Mem, Fetch>>;
}

impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, Depth> Interpreter
    for InterpreterState<Code, Mem, FetchGoingBack<Depth>>
where
    InterpreterState<Code, Mem, GoingBack<Code::Read, Depth>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code, Mem, GoingBack<Code::Read, Depth>>>;
}

// Go back to startloop (depth=0, return to execution)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, GoingBack<StartLoop, Zero>>
where
    InterpreterState<Code::MoveRight, Mem, Fetch>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveRight, Mem, Fetch>>;
}

// Go back to startloop (depth>0, decrease depth)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<StartLoop, Succ<I>>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>>;
}

// Go back to endloop (increase depth)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<EndLoop, I>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<Succ<I>>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<Succ<I>>>>;
}

// Go back to halt (code overran, halt)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<Halt, I>>
{
    type FinalState = InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>;
}

// Go back to increment, decrement, move right, move left (all no-op)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<Increment, I>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<Decrement, I>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<MoveRight, I>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, GoingBack<MoveLeft, I>>
where
    InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>: Interpreter,
{
    type FinalState = IRec<InterpreterState<Code::MoveLeft, Mem, FetchGoingBack<I>>>;
}

type Run<Program> =
    IRec<InterpreterState<TDoubleStack<Nil, Program>, TDoubleStack<Nil, Nil>, Fetch>>;

type Program = Cons<Increment, Cons<MoveRight, Cons<Increment, Cons<Increment, Cons<StartLoop, Cons<Decrement, Cons<EndLoop, Nil>>>>>>>;

fn main() {
    println!("{}", type_name::<Run<Program>>());
}
