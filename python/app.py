import gmap
import jsonschema
from quart import Quart, abort, render_template, request, url_for
import traceback

def create_app(solvers):
    app = Quart(__name__)

    @app.route('/')
    async def puzzle():
        return await render_template('index.html', solve_endpoint=url_for('solve', solver='custom'))

    layer_schema = {
        'type': 'object',
        'properties': {
            'name': {'type': 'string'},
            'type': {
                'type': 'string',
                'pattern': r'^(string|enum)$',
            },
            'data': gmap.OrbitDict.schema,
        },
        'required': ['name', 'type', 'data'],
    }
    solve_req_schema = {
        'type': 'object',
        'properties': {
            'graph': gmap.GMap.schema,
            'layers': {
                'type': 'array',
                'items': layer_schema,
            },
        },
        'required': ['graph', 'layers'],
    }
    solve_resp_schema = {
        'type': 'object',
        'properties': {
            'layers': {
                'type': 'array',
                'items': layer_schema,
            },
        },
        'required': ['layers'],
    }

    @app.route('/solve/<solver>', methods=['POST'])
    async def solve(solver):
        try:
            s = solvers[solver]
        except KeyError:
            return {'error': 'solver not found'}, 400

        data = await request.get_json()
        if not data:
            return {'error': 'missing json data'}, 400
        try:
            jsonschema.validate(data, schema=solve_req_schema)
            graph = gmap.GMap.deserialize(data['graph'])
            layers = [
                {
                    'name': v['name'],
                    'type': v['type'],
                    'data': gmap.OrbitDict.deserialize(v['data']),
                }
                for v in data['layers']
            ]
            extra = data.get('extra')
        except (ValueError, KeyError, jsonschema.ValidationError) as e:
            traceback.print_exc()
            return {'error': repr(e)}, 400

        output = s(graph, layers, extra=extra)

        output_json = {
            'layers': {
                v['name']: {
                    'type': v['type'],
                    'data': v['data'].serialize(),
                }
                for v in output['layers']
            },
            'extra': output['extra'],
        }
        jsonschema.validate(output_json, schema=solve_resp_schema)
        return output_json
        

    return app
