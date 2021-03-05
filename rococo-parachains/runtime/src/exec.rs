//TODO License

//! Block executive to be used by relay chain validators when validating parachain blocks built
//! with the nimubs consensus family.

// This trait will probably move to frame-support soon.
use frame_executive::ExecuteBlock;
use sp_api::{BlockT, HeaderT};
use log::info;
use sp_runtime::generic::DigestItem;

/// Block executive to be used by relay chain validators when validating parachain blocks built
/// with the nimubs consensus family.
///
/// This will strip the seal digest, and confirm that only a single such digest exists.
/// It then passes the pre-block to the inner executive which will likely be the normal FRAME
/// executive as it is run on the parachain itself.
/// (Aspitational) Finally it puts the original digest back on and confirms the blocks match
///
/// Essentially this contains the logic of the verifier and the normal executive.
/// TODO Degisn improvement:
/// Can we share code with the verifier?
/// Can this struct take a verifier as an associated type?
pub struct BlockExecutor<T, I>(sp_std::marker::PhantomData<(T, I)>);

impl<Block, T, I> ExecuteBlock<Block> for BlockExecutor<T, I>
where
	Block: BlockT,
	I: ExecuteBlock<Block>,
{
	fn execute_block(block: Block) {
		let (mut header, extrinsics) = block.deconstruct();

        info!("In hacked Executive. Initial digests are {:?}", header.digest());

		// Set the seal aside for checking. Currently there is nothing to check.
		let seal = header
			.digest_mut()
			.logs //hmmm how does the compiler know that my digest type has a logs field?
			.pop()
			.expect("Seal digest is present and is last item");

		info!("In hacked Executive. digests after stripping {:?}", header.digest());
		info!("The seal we got {:?}", seal);

		I::execute_block(Block::new(header, extrinsics));

		// TODO The verifier does additional work here. I wonder if it's important that we get the
		// validators doing that.
	}
}
