const { Oso } = require('oso');

const { Organization, Repository, Role, User } = require('./app');

const oso = new Oso();

oso.registerClass(Organization);
oso.registerClass(Repository);
oso.registerClass(User);

beforeEach(() => oso.loadFiles(["main.polar"]));
afterEach(() => oso.clearRules());

const alphaAssociation = new Organization("Alpha Association");
const betaBusiness = new Organization("Beta Business");

const affineTypes = new Repository("Affine Types", alphaAssociation);
const allocator = new Repository("Allocator", alphaAssociation);
const bubbleSort = new Repository("Bubble Sort", betaBusiness);
const benchmarks = new Repository("Benchmarks", betaBusiness);

const ariana = new User("Ariana");
const bhavik = new User("Bhavik");

ariana.assignRoleForResource("owner", alphaAssociation);
bhavik.assignRoleForResource("contributor", bubbleSort);
bhavik.assignRoleForResource("maintainer", benchmarks);

test('policy', async () => {
    await expect(oso.authorize(ariana, "read", affineTypes)).resolves.toBeUndefined();
    await expect(oso.authorize(ariana, "push", affineTypes)).resolves.toBeUndefined();
    await expect(oso.authorize(ariana, "read", allocator)).resolves.toBeUndefined();
    await expect(oso.authorize(ariana, "push", allocator)).resolves.toBeUndefined();
    await expect(oso.authorize(ariana, "read", bubbleSort)).rejects.toThrow('404');
    await expect(oso.authorize(ariana, "push", bubbleSort)).rejects.toThrow('404');
    await expect(oso.authorize(ariana, "read", benchmarks)).rejects.toThrow('404');
    await expect(oso.authorize(ariana, "push", benchmarks)).rejects.toThrow('404');

    await expect(oso.authorize(bhavik, "read", affineTypes)).rejects.toThrow('404');
    await expect(oso.authorize(bhavik, "push", affineTypes)).rejects.toThrow('404');
    await expect(oso.authorize(bhavik, "read", allocator)).rejects.toThrow('404');
    await expect(oso.authorize(bhavik, "push", allocator)).rejects.toThrow('404');
    await expect(oso.authorize(bhavik, "read", bubbleSort)).resolves.toBeUndefined();
    await expect(oso.authorize(bhavik, "push", bubbleSort)).rejects.toThrow('403');
    await expect(oso.authorize(bhavik, "read", benchmarks)).resolves.toBeUndefined();
    await expect(oso.authorize(bhavik, "push", benchmarks)).resolves.toBeUndefined();
});
