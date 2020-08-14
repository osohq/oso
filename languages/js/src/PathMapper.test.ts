import { PathMapper } from './PathMapper';
import { Http } from './Http';
import { Oso } from './Oso';
import { Widget } from '../test/classes';

describe('#map', () => {
  test('extracts matches into an object', () => {
    const pm = new PathMapper('/widget/{id}');
    expect(pm.map('/widget/12')).toStrictEqual({ id: '12' });
    expect(pm.map('/widget/12/frob')).toStrictEqual({});
  });
});

describe('PathMapper + Http', () => {
  test('can map Http resources', () => {
    const o = new Oso();
    o.registerClass(Widget);
    o.loadStr(`
      allow(actor, "get", _: Http{path: path}) if
          new PathMapper("/widget/{id}").map(path) = x and
          allow(actor, "get", new Widget(x.id));
      allow(_actor, "get", widget: Widget) if widget.id = "12";
    `);
    const widget12 = new Http('host', '/widget/12', {});
    let allowed = o.isAllowed('sam', 'get', widget12);
    expect(allowed).toBe(true);
    const widget13 = new Http('host', '/widget/13', {});
    allowed = o.isAllowed('sam', 'get', widget13);
    expect(allowed).toBe(false);
  });
});
