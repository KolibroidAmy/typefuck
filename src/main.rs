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

struct IOState<Input, Output>(PhantomData<Input>, PhantomData<Output>, Uninhabited);

struct MoveRight(Uninhabited);
struct MoveLeft(Uninhabited);
struct Increment(Uninhabited);
struct Decrement(Uninhabited);
struct EndLoop(Uninhabited);
struct StartLoop(Uninhabited);
struct Getch(Uninhabited);
struct Putch(Uninhabited);
struct Halt(Uninhabited);

struct FetchGoingBack<Depth>(PhantomData<Depth>, Uninhabited);
struct GoingBack<Saw, Depth>(PhantomData<Saw>, PhantomData<Depth>, Uninhabited);
struct Execute<Inst>(PhantomData<Inst>, Uninhabited);
struct Fetch(Uninhabited);
struct LoopCompare<Num>(PhantomData<Num>, Uninhabited);

struct InterpreterState<
    Code: InfTape<Halt>,
    Mem: InfTape<Zero>,
    In: InfTape<Zero>,
    Out: InfTape<Zero>,
    State,
>(
    PhantomData<Code>,
    PhantomData<Mem>,
    PhantomData<In>,
    PhantomData<Out>,
    PhantomData<State>,
    Uninhabited,
);

trait Interpreter {
    type Output;
}

type IRec<I: Interpreter> = <I as Interpreter>::Output;

// Fetch
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Fetch>
where
    InterpreterState<Code, Mem, In, Out, Execute<Code::Read>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code, Mem, In, Out, Execute<Code::Read>>>;
}

// Halt
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<Halt>>
{
    type Output = Out;
}

// Move right
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<MoveRight>>
where
    InterpreterState<Code::MoveRight, Mem::MoveRight, In, Out, Fetch>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveRight, Mem::MoveRight, In, Out, Fetch>>;
}

// Move left
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<MoveLeft>>
where
    InterpreterState<Code::MoveRight, Mem::MoveLeft, In, Out, Fetch>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveRight, Mem::MoveLeft, In, Out, Fetch>>;
}

// Increment
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<Increment>>
where
    Mem::Read: TNumber,
    InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Incr>, In, Out, Fetch>:
        Interpreter,
{
    type Output = IRec<
        InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Incr>, In, Out, Fetch>,
    >;
}

// Decrement
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<Decrement>>
where
    Mem::Read: TNumber,
    InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Decr>, In, Out, Fetch>:
        Interpreter,
{
    type Output = IRec<
        InterpreterState<Code::MoveRight, Mem::Write<<Mem::Read as TNumber>::Decr>, In, Out, Fetch>,
    >;
}

// Getch
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<Getch>>
where
    InterpreterState<Code::MoveRight, Mem::Write<In::Read>, In::MoveRight, Out, Fetch>: Interpreter,
{
    type Output =
        IRec<InterpreterState<Code::MoveRight, Mem::Write<In::Read>, In::MoveRight, Out, Fetch>>;
}

// Putch
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<Putch>>
where
    InterpreterState<Code::MoveRight, Mem, In, <Out::Write<Mem::Read> as InfTape<Zero>>::MoveRight, Fetch>:
        Interpreter,
{
    type Output =
        IRec<InterpreterState<Code::MoveRight, Mem, In, <Out::Write<Mem::Read> as InfTape<Zero>>::MoveRight, Fetch>>;
}

// Execute startloop (no-op)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<StartLoop>>
where
    InterpreterState<Code::MoveRight, Mem, In, Out, Fetch>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveRight, Mem, In, Out, Fetch>>;
}

// Execute endloop (begin compare)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, Execute<EndLoop>>
where
    InterpreterState<Code, Mem, In, Out, LoopCompare<Mem::Read>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code, Mem, In, Out, LoopCompare<Mem::Read>>>;
}

// Loop value != 0 (begin loop)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, LoopCompare<Succ<I>>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<Zero>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<Zero>>>;
}

// Loop value == 0 (no-op)
impl<
        Code: InfTape<Halt>,
        Mem: InfTape<Zero, Read = Zero>,
        In: InfTape<Zero>,
        Out: InfTape<Zero>,
    > Interpreter for InterpreterState<Code, Mem, In, Out, LoopCompare<Zero>>
where
    InterpreterState<Code::MoveRight, Mem, In, Out, Fetch>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveRight, Mem, In, Out, Fetch>>;
}

impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, Depth>
    Interpreter for InterpreterState<Code, Mem, In, Out, FetchGoingBack<Depth>>
where
    InterpreterState<Code, Mem, In, Out, GoingBack<Code::Read, Depth>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code, Mem, In, Out, GoingBack<Code::Read, Depth>>>;
}

// Go back to startloop (depth=0, return to execution)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<StartLoop, Zero>>
where
    InterpreterState<Code::MoveRight, Mem, In, Out, Fetch>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveRight, Mem, In, Out, Fetch>>;
}

// Go back to startloop (depth>0, decrease depth)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<StartLoop, Succ<I>>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}

// Go back to endloop (increase depth)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<EndLoop, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<Succ<I>>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<Succ<I>>>>;
}

// Go back to halt (code overran, halt)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<Halt, I>>
{
    type Output = Out;
}

// Go back to increment, decrement, move right, move left (all no-op)
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<Increment, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<Decrement, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<MoveRight, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<MoveLeft, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<Getch, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}
impl<Code: InfTape<Halt>, Mem: InfTape<Zero>, In: InfTape<Zero>, Out: InfTape<Zero>, I> Interpreter
    for InterpreterState<Code, Mem, In, Out, GoingBack<Putch, I>>
where
    InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>: Interpreter,
{
    type Output = IRec<InterpreterState<Code::MoveLeft, Mem, In, Out, FetchGoingBack<I>>>;
}

type Run<Program, Input> = IRec<
    InterpreterState<
        TDoubleStack<Nil, Program>,
        TDoubleStack<Nil, Nil>,
        TDoubleStack<Nil, Input>,
        TDoubleStack<Nil, Nil>,
        Fetch,
    >,
>;


macro_rules! tcons_list {
    () => {
        Nil
    };
    ($x: ty $(,$xs:ty)* $(,)?) => {
        Cons<$x, tcons_list!($($xs),*)>
    };
}

type Program = tcons_list!(Getch, StartLoop, Putch, Getch, EndLoop);
type Input = tcons_list!(Succ<Succ<Succ<Zero>>>);

type FinalOutput = Run<Program, Input>;


trait Unimplemented {}
fn cause_type_error<A: Unimplemented>() {}


fn main() {
    cause_type_error::<FinalOutput>()
}
