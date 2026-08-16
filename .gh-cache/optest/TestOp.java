import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

public class TestOp {
    public static void main(String[] args) {
        AABB global = new AABB(-1000, -64, -1000, 1000, 320, 1000);
        AABB street = new AABB(595, 72, 729, 606, 88, 738);
        VoxelShape free = Shapes.join(Shapes.create(global), Shapes.create(street), BooleanOp.ONLY_FIRST);
        AABB house = new AABB(596, 72, 729, 603, 79, 738);
        VoxelShape h = Shapes.create(house.deflate(0.25));
        boolean reject = Shapes.joinIsNotEmpty(free, h, BooleanOp.ONLY_SECOND);
        System.out.println("house fully inside expanded street -> reject=" + reject);
        // house partially overlapping street (extends beyond)
        AABB house2 = new AABB(600, 72, 729, 610, 79, 738);
        boolean reject2 = Shapes.joinIsNotEmpty(free, Shapes.create(house2.deflate(0.25)), BooleanOp.ONLY_SECOND);
        System.out.println("house partially outside street -> reject=" + reject2);
        // house next to street (x adjacent, sharing boundary)
        AABB house3 = new AABB(606, 72, 729, 614, 79, 738);
        boolean reject3 = Shapes.joinIsNotEmpty(free, Shapes.create(house3.deflate(0.25)), BooleanOp.ONLY_SECOND);
        System.out.println("house adjacent to street (boundary) -> reject=" + reject3);
    }
}
